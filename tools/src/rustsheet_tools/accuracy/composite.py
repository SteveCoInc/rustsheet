"""Math references for composite cells (T-3993, decision 0297; SPEC-0029 "Math references for eval
cells").

A composite cell's formula composes calls of functions that have a Gate A definition: a call nested
in another call's arguments (`=F.INV(F.DIST(0.99,398,150579,TRUE),398,150579)`), or arithmetic
between call results (`=BINOM.DIST(k1,n,p,TRUE)-BINOM.DIST(k2,n,p,TRUE)`). Its math value is the
outer node's math at the doubles its inner nodes round to (decision 0297's option (b)): each inner
call is its 50-digit math rounded to the nearest double (a subnormal call result is 0, numbers.md
"Subnormal results become 0"), each inner `+ - * /` is the exact operation on its operands' doubles
rounded the same way (a subnormal `*` or `/` is 0), and the outermost node is given to 50 digits.
That is what a pipeline that rounds every step correctly gives before its last rounding. Option (a),
the exact composition with no inner rounding, is kept as `rounded=False` for the decision's scoring.

A part of the formula with no call in it (an argument, an operand) is read as the one-call cells
read it (`parse_arg`, `arithmetic`): literals, exact-input cells and arithmetic over them with
Excel's operator model. `^` with a call result as an operand is not read (Excel's `^` is not
correctly rounded, numbers.md), nor is anything the fit lab's arithmetic does not read (`&`,
comparisons, a range of a call's results).
"""

from __future__ import annotations

import re
from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

import mpmath

from rustsheet_tools.accuracy import literal
from rustsheet_tools.accuracy.model import REGISTRY, Definition, DomainError, NoValueError, Param, Unfit
from rustsheet_tools.accuracy.sampler import math_arg
from rustsheet_tools.accuracy.tables import DOUBLE_MAX, MathValue, compute_value
from rustsheet_tools.fitlab.exact import X87
from rustsheet_tools.fitlab.sweep import (
    ARITH_TOKEN,
    MAX_NESTING,
    UnparsedError,
    Workbook,
    arithmetic,
    cell_number,
    parse_arg,
    parse_matrix_arg,
    split_args,
    unquoted,
)
from rustsheet_tools.fitlab.sweep import (
    Arg as RawArg,
)

# A function call's name and opening parenthesis (`F.DIST(`, `LOG10 (`).
_CALL_START = re.compile(r"\s*([A-Za-z_][\w.]*)\s*\(")
_SMALLEST_NORMAL = 2.0**-1022


class CompositeSkipError(NoValueError):
    """A composite whose inner value does not fit the outer parameter (a non-integer result for an
    integer parameter); the message is the sidecar's `why`."""


@dataclass(frozen=True)
class Const:
    """A call-free part: its double, as the one-call cells read it."""

    value: float


@dataclass(frozen=True)
class Neg:
    """A unary minus of a part that holds a call."""

    operand: Node


@dataclass(frozen=True)
class Op:
    """A binary `+ - * /` with a call on at least one side."""

    op: str
    left: Node
    right: Node


@dataclass(frozen=True)
class Call:
    """A call of a function with a definition. An argument is a fitted literal argument
    (`eval_math.fit_arg`) or a part that holds a call."""

    definition: Definition
    args: tuple[literal.Arg | Node, ...]


Node = Const | Neg | Op | Call
# How a call-free argument is fitted to its parameter (`eval_math.fit_arg`): the argument, or why
# it does not fit (an `Unfit`; a text argument is a plain str).
FitArg = Callable[[Param, Any], literal.Arg | Unfit]


class _Parser:
    """A recursive-descent parser over one formula (see the module doc); a call-free part is a
    span of the text, read by `arithmetic` once its extent is known."""

    def __init__(self, text: str, sheet: str, book: Workbook, fit: FitArg) -> None:
        self.text = text
        self.pos = 0
        self.sheet = sheet
        self.book = book
        self.fit = fit
        self.undefined: list[str] = []
        self.calls = 0

    def fail(self, what: str) -> UnparsedError:
        return UnparsedError(f"{what} in {self.text!r}")

    def peek(self) -> str | None:
        m = ARITH_TOKEN.match(self.text, self.pos)
        return m.group("op") if m else None

    def skip_op(self) -> None:
        m = ARITH_TOKEN.match(self.text, self.pos)
        if m is None:
            raise self.fail("unexpected end")
        self.pos = m.end()

    def node(self, part: tuple[int, int] | Node) -> Node:
        """A call-free span as a Const, any other part as is."""
        if not isinstance(part, tuple):
            return part
        value = arithmetic(self.text[part[0] : part[1]], lambda ref: cell_number(ref, self.sheet, self.book))
        return Const(value + 0.0)  # Excel has no negative zero (`parse_arg`)

    def binary(
        self, op: str, left: tuple[int, int] | Node, right: tuple[int, int] | Node
    ) -> tuple[int, int] | Node:
        if isinstance(left, tuple) and isinstance(right, tuple):
            return (left[0], right[1])
        if op == "^":
            raise self.fail("a power of a call result")
        return Op(op, self.node(left), self.node(right))

    def additive(self, depth: int) -> tuple[int, int] | Node:
        part = self.multiplicative(depth)
        while (op := self.peek()) in ("+", "-"):
            self.skip_op()
            part = self.binary(op, part, self.multiplicative(depth))
        return part

    def multiplicative(self, depth: int) -> tuple[int, int] | Node:
        part = self.power(depth)
        while (op := self.peek()) in ("*", "/"):
            self.skip_op()
            part = self.binary(op, part, self.power(depth))
        return part

    def power(self, depth: int) -> tuple[int, int] | Node:
        part = self.primary(depth)
        while self.peek() == "^":
            self.skip_op()
            part = self.binary("^", part, self.primary(depth))
        return part

    def primary(self, depth: int) -> tuple[int, int] | Node:
        if depth > MAX_NESTING:
            raise self.fail("nesting too deep")
        start = self.pos
        if (call := _CALL_START.match(self.text, self.pos)) is not None:
            return self.call(call, depth)
        m = ARITH_TOKEN.match(self.text, self.pos)
        if m is None:
            raise self.fail("unexpected input")
        self.pos = m.end()
        if m.group("op") in ("+", "-"):
            inner = self.primary(depth + 1)
            if isinstance(inner, tuple):
                return (start, inner[1])
            return Neg(inner) if m.group("op") == "-" else inner
        if m.group("op") == "(":
            inner = self.additive(depth + 1)
            if self.peek() != ")":
                raise self.fail("unbalanced parentheses")
            self.skip_op()
            return (start, self.pos) if isinstance(inner, tuple) else inner
        if m.group("op") is not None:
            raise self.fail(f"unexpected {m.group('op')!r}")
        return (start, self.pos)  # a number or a cell reference

    def call(self, m: re.Match[str], depth: int) -> Node:
        name = m.group(1).upper()
        close = _closing(self.text, m.end())
        if close is None:
            raise self.fail("unbalanced parentheses")
        texts = split_args(self.text[m.end() : close])
        self.pos = close + 1
        self.calls += 1
        args: list[RawArg | Node] = []
        for text in texts:
            try:  # a call-free argument, read as the one-call cells read it
                args.append(parse_arg(text, self.sheet, self.book, cell_refs=True))
            except UnparsedError:
                inner = _Parser(text, self.sheet, self.book, self.fit)
                part = inner.additive(depth + 1)
                if inner.pos != len(text.rstrip()):
                    raise self.fail("trailing input") from None
                if isinstance(part, tuple):
                    raise
                self.undefined += inner.undefined
                self.calls += inner.calls
                args.append(part)
        definition = REGISTRY.get(name)
        if definition is None:
            self.undefined.append(name)
            return Const(0.0)  # a stand-in: the candidate is skipped (`parse`)
        try:
            params = definition.params_for(len(args))
        except ValueError:
            raise self.fail(f"argument count of {name}") from None
        fitted: list[literal.Arg | Node] = []
        for param, arg, text in zip(params, args, texts, strict=True):
            if isinstance(arg, Const | Neg | Op | Call):
                if param.kind not in ("real", "int"):
                    raise self.fail(f"a call result for {name}'s {param.kind} parameter")
                fitted.append(arg)
                continue
            # a matrix keeps its rows, which `parse_arg` flattens (T-4025)
            shaped = parse_matrix_arg(text, self.sheet, self.book) if param.kind == "matrix" else arg
            value = self.fit(param, shaped)
            if isinstance(value, Unfit):
                raise self.fail(f"{name}: {value}")
            fitted.append(value)
        return Call(definition, tuple(fitted))


def _closing(text: str, start: int) -> int | None:
    """The index of the `)` that closes the parenthesis opened just before `start`."""
    for i, ch, depth in unquoted(text, start):
        if depth < 0:
            return i if ch == ")" else None
    return None


@dataclass(frozen=True)
class Composite:
    """A parsed composite formula, or why it gets no math value."""

    root: Node | None
    why: str | None


def parse(text: str, sheet: str, book: Workbook, fit: FitArg) -> Composite | None:
    """The composite tree of a formula `=...` that holds a call and is not one call (the one-call
    path reads those), or None when it is no composite this module reads."""
    if not text.startswith("="):
        return None
    parser = _Parser(text[1:], sheet, book, fit)
    try:
        part = parser.additive(0)
        if parser.pos != len(parser.text.rstrip()):
            return None
    except (UnparsedError, ValueError, OverflowError):
        return None
    if isinstance(part, tuple) or parser.calls == 0:
        return None
    if parser.undefined:
        return Composite(None, f"no definition for {', '.join(sorted(set(parser.undefined)))}")
    return Composite(part, None)


def _double(value: mpmath.mpf, flush: bool) -> float:
    """The double `value` rounds to; with `flush`, a subnormal is 0 (a call's result, numbers.md
    "Subnormal results become 0"). NoValueError beyond the double range."""
    if not mpmath.isfinite(value) or abs(value) > DOUBLE_MAX:
        msg = "an inner value beyond the double range"
        raise NoValueError(msg)
    nearest = float(value)
    return 0.0 if flush and 0 < abs(nearest) < _SMALLEST_NORMAL else nearest + 0.0


def _excel_double(node: Node) -> float:
    """The double Excel's pipeline gives an inner node under option (b): a call's math rounded to
    the nearest double, and `+ - * /` over its operands' doubles with Excel's operator model, the
    one `arithmetic` applies to call-free parts (x87 double rounding, a subnormal `*` or `/` is 0)."""
    if isinstance(node, Const):
        return node.value
    if isinstance(node, Neg):
        return -_excel_double(node.operand) + 0.0
    if isinstance(node, Call):
        return _double(value(node, rounded=True), flush=True)
    left, right = _excel_double(node.left), _excel_double(node.right)
    if node.op == "/" and right == 0:
        msg = "#DIV/0!"
        raise DomainError(msg)
    try:
        result = {"+": X87.add, "-": X87.sub, "*": X87.mul, "/": X87.div}[node.op](left, right)
    except OverflowError:
        msg = "an inner value beyond the double range"
        raise NoValueError(msg) from None
    return 0.0 if node.op in "*/" and 0 < abs(result) < _SMALLEST_NORMAL else result


def _operand(node: Node, *, rounded: bool) -> mpmath.mpf:
    """An inner node's value as its parent reads it: its double under option (b)."""
    return mpmath.mpf(_excel_double(node)) if rounded else value(node, rounded=False)


def value(node: Node, *, rounded: bool) -> mpmath.mpf:
    """The math value of the outermost `node` at the current precision: the exact operation or
    call on its inner nodes' doubles (`rounded`, decision 0297's option (b)), or the exact
    composition (option (a)). DomainError when a call is outside its domain or a divisor is 0."""
    if isinstance(node, Const):
        return mpmath.mpf(node.value)
    if isinstance(node, Neg):
        return -value(node.operand, rounded=rounded)
    if isinstance(node, Op):
        left, right = _operand(node.left, rounded=rounded), _operand(node.right, rounded=rounded)
        if node.op == "/" and right == 0:
            msg = "#DIV/0!"
            raise DomainError(msg)
        if node.op == "+":
            return left + right
        if node.op == "-":
            return left - right
        return left * right if node.op == "*" else left / right
    params = node.definition.params_for(len(node.args))
    args = [_call_arg(p, a, rounded=rounded) for p, a in zip(params, node.args, strict=True)]
    return node.definition.evaluate(args)


def _call_arg(param: Param, arg: literal.Arg | Node, *, rounded: bool) -> Any:
    """One argument of a call as its definition reads it. An inner node for an integer parameter
    is its double, which must be an integer (whether Excel truncates is per function, as in
    `eval_math.fit_int`)."""
    if not isinstance(arg, Const | Neg | Op | Call):
        return math_arg(param, arg)
    if param.kind != "int":
        return _operand(arg, rounded=rounded)
    double = _excel_double(arg) if rounded else _double(value(arg, rounded=False), flush=True)
    if not double.is_integer():
        msg = "a non-integer inner value for an integer parameter"
        raise CompositeSkipError(msg)
    return mpmath.mpf(int(double))


def compute(root: Node, *, rounded: bool = True) -> MathValue:
    """The composite's math value (`tables.compute_value`: dps 80, checked against dps 60)."""
    return compute_value(lambda: value(root, rounded=rounded))
