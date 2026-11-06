"""Sweep inputs: the cells of an oracle-filled `.case.json` that a fit scores.

A sweep is a case plus its oracle sidecar (Excel's values), both JSON, located through the
harness's own discovery (`rustsheet-eval cases`, `snapshot.suite_cases`); no xlsx is parsed.
Cells are selected by a regex over the formula text whose first group is the argument list.
Arguments are numeric literals, TRUE/FALSE, string literals, array literals of those (`{1,2;3,4}`,
row-major), `DATE(y,m,d)` with integer literals (a 1900-system serial), references (`A1`, `$Z$1`,
`Data!B2`, `D1:D10`) to constant cells of the case or to formula cells with an oracle value, and
arithmetic over numeric literals (`0.05/12`, `1+26955297717044*2^-52`), which is evaluated with
Excel's operator model (`arithmetic`; with `parse_arg`'s `cell_refs`, that arithmetic may also read
single cells holding a number, `A112*A112`, for the math sidecars, T-3924), and a nested call whose text is the whole formula of a cell
of the fit's `calls` cases (`PMT(0.2,1,10000)` in `RATE(1,PMT(0.2,1,10000),10000)`), which is
Excel's value of that cell: the outer kernel is then scored on Excel's own bits of the inner call.
A cell whose arguments are anything else (another function call) is reported as unparsed. A pattern
with one group is an argument list split at top-level commas; a pattern with several groups gives one
argument per group that took part in the match.
"""

from __future__ import annotations

import datetime
import json
import math
import re
from collections.abc import Callable
from dataclasses import dataclass, field
from decimal import ROUND_DOWN, Decimal
from pathlib import Path
from typing import Any

from xlsxwriter.utility import xl_rowcol_to_cell

from rustsheet_tools.fitlab.exact import DOUBLE, DOUBLE_MIN_EXP, X87, exact, round_to_bits
from rustsheet_tools.snapshot import is_undecoded, suite_cases
from rustsheet_tools.spec2xlsx import parse_a1

# What a candidate receives for one argument: a number, a bool, a string, an empty cell (None)
# or, for a range, the list of those values in row-major order.
Arg = float | bool | str | None | list[float | bool | str | None]
# A cell reference's text inside arithmetic -> that cell's number (`arithmetic`'s `resolve`).
Resolve = Callable[[str], float]


def numeric_arg(a: Arg) -> float:
    """`a` as a number for a kernel candidate; TypeError for anything else (a bool, text, an
    empty cell or a range), which the scorer counts as that candidate's miss on the cell."""
    if isinstance(a, bool) or not isinstance(a, float):
        msg = f"non-numeric argument {a!r}"
        raise TypeError(msg)
    return a


def number_literal(text: str) -> float:
    """A formula's number literal as Excel reads it: cut (not rounded) to 15 significant digits,
    so `6.727499949325611` is 6.72749994932561 (numbers.md, "Formula number literals are read at
    15 significant digits"; probes/xirr-exact-root-sign, T-2328)."""
    value = Decimal(text)
    if value.is_finite() and value != 0:
        value = value.quantize(Decimal(1).scaleb(value.adjusted() - 14), rounding=ROUND_DOWN)
    return float(value)


_UNSIGNED = r"(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?"
_NUMBER = re.compile(rf"[+-]?{_UNSIGNED}")

_COMPLEX_TEXT = re.compile(
    rf"(?P<re>[+-]?{_UNSIGNED})?"
    rf"(?:(?P<sign>[+-]?)(?P<im>{_UNSIGNED})?(?P<unit>[ij]))?"
)


def parse_complex_text(text: str) -> tuple[float, float]:
    """An IM* function's text result (`1.5-2i`, `-0.5j`, `i`, `3`, `1E+307+1.6E+308i`) as its
    real and imaginary parts; ValueError for anything else. The imaginary part needs its own sign
    when a real part precedes it, so `1.5i` is 0+1.5i, not 1.5+1i."""
    m = _COMPLEX_TEXT.fullmatch(text)
    if m is None or (m["re"] is None and m["unit"] is None):
        msg = f"not a complex number: {text!r}"
        raise ValueError(msg)
    if m["unit"] is None:
        return float(m["re"]), 0.0
    if m["re"] is not None and not m["sign"]:
        return 0.0, float(m["re"])  # `1.5i`: the greedy real part took the coefficient
    im = float(m["im"]) if m["im"] is not None else 1.0
    return (float(m["re"]) if m["re"] is not None else 0.0), (-im if m["sign"] == "-" else im)


_SHEET = r"'(?:[^']|'')+'|[A-Za-z_][\w.]*"
_A1 = r"\$?[A-Za-z]{1,3}\$?\d+"
_REF = re.compile(rf"(?:(?P<sheet>{_SHEET})!)?(?P<first>{_A1})(?:\s*:\s*(?P<last>{_A1}))?")
_MAX_RANGE_CELLS = 1_000_000
# One cell reference inside arithmetic (`A1249`, `Calc!$A$2`), never a function name such as
# `LOG10(` or the start of a range.
_CELL_TOKEN = rf"(?:(?:{_SHEET})!)?{_A1}(?![\w.$:!]|\s*[(:])"
ARITH_TOKEN = re.compile(rf"\s*(?:(?P<num>{_UNSIGNED})|(?P<ref>{_CELL_TOKEN})|(?P<op>[-+*/^()]))")
_MAX_EXACT_EXPONENT = 1100  # beyond the double range either way
_OPERATORS = frozenset("+-*/^()")
MAX_NESTING = 50  # parentheses and unary signs: a bound on the parser's recursion (directive 5)
_CALL = re.compile(r"=[A-Za-z][\w.]*\([^()]*\)")  # one call over literals: `=PMT(0.2,1,10000)`
_DATE = re.compile(r"DATE\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)", re.IGNORECASE)
EPOCH = datetime.date(1899, 12, 30)  # serial 61 = 1900-03-01 (Excel's phantom 1900-02-29 is 60)


class UnparsedError(ValueError):
    """An argument the sweep reader does not evaluate (e.g. a nested expression)."""


@dataclass(frozen=True)
class Cell:
    """One scored cell: where it is, its formula, parsed arguments and Excel's value."""

    case: str
    sheet: str
    address: str
    formula: str
    args: tuple[Arg, ...]
    expected: float  # NaN when Excel's value is an error (`error` names it); a complex text's real part
    error: str | None = None  # Excel's error value (`#NUM!`) when the cell holds one
    expected_im: float | None = None  # a complex text's imaginary part (`Fit.complex_text`), else None

    @property
    def key(self) -> str:
        """`Sheet!A1`, the key `rustsheet-eval show` prints."""
        return f"{self.sheet}!{self.address}"


@dataclass
class Sweep:
    """The selected cells of one or more cases, plus what was left out and why."""

    cells: list[Cell] = field(default_factory=list)
    unparsed: list[str] = field(default_factory=list)  # "case Sheet!A1: reason"
    non_numeric: int = 0  # Excel's value is text, a bool (or an error unless the fit scores errors)
    excluded: int = 0  # matched the pattern but the fit's `where` filter left it out


def split_args(text: str) -> list[str]:
    """Splits an argument list at top-level commas (outside parentheses, braces and strings)."""
    parts: list[str] = []
    depth, start, quoted = 0, 0, False
    for i, ch in enumerate(text):
        if ch == '"':
            quoted = not quoted
        elif quoted:
            continue
        elif ch in "({":
            depth += 1
        elif ch in ")}":
            depth -= 1
        elif ch == "," and depth == 0:
            parts.append(text[start:i])
            start = i + 1
    parts.append(text[start:])
    return [p.strip() for p in parts] if text.strip() else []


@dataclass(frozen=True)
class Workbook:
    """A case's sheets: spec cell values and the oracle's values for formula cells."""

    sheets: dict[str, dict[str, Any]]
    oracle: dict[str, dict[str, Any]]
    calls: dict[str, float] = field(default_factory=dict)  # `call_key` -> Excel's value

    def value(self, sheet: str, address: str) -> float | bool | str | None:
        """A referenced cell's value: a constant from the case, or Excel's value of a formula."""
        cells = self.sheets.get(sheet)
        if cells is None:
            msg = f"no sheet {sheet!r}"
            raise UnparsedError(msg)
        raw = cells.get(address)
        if isinstance(raw, str) and raw.startswith("="):
            return _oracle_value(self.oracle.get(sheet, {}).get(address), f"{sheet}!{address}")
        if isinstance(raw, dict):
            return _oracle_value(raw, f"{sheet}!{address}")
        if raw is None or isinstance(raw, bool | str):
            return raw
        if isinstance(raw, int | float):
            return float(raw)
        msg = f"unsupported cell value at {sheet}!{address}"
        raise UnparsedError(msg)


def _oracle_value(entry: Any, where: str) -> float | bool | str | None:
    """A `{"t": .., "v": ..}` value (oracle sidecar or typed case cell) as an argument."""
    if not isinstance(entry, dict):
        msg = f"{where} has no oracle value"
        raise UnparsedError(msg)
    tag, value = entry.get("t"), entry.get("v")
    if tag == "n":
        return float(entry["v"])
    if tag == "b":
        return bool(value)
    if tag == "s":
        return str(value)
    msg = f"{where} is an error ({value})"
    raise UnparsedError(msg)


def _sheet_name(raw: str | None, default: str) -> str:
    if raw is None:
        return default
    return raw[1:-1].replace("''", "'") if raw.startswith("'") else raw


def _tokens(text: str) -> list[str]:
    """`text` split into unsigned numbers, single-cell references and the operators
    `+ - * / ^ ( )`, whitespace dropped; UnparsedError at anything else (a function name, a range,
    a comma)."""
    out: list[str] = []
    pos, end = 0, len(text.rstrip())
    while pos < end:
        m = ARITH_TOKEN.match(text, pos)
        if not m:
            msg = f"cannot evaluate argument {text!r}"
            raise UnparsedError(msg)
        out.append(m.group("num") or m.group("ref") or m.group("op"))
        pos = m.end()
    return out


def _exact_power(base: float, power: float) -> float:
    """`base^power` when it is an integer power whose exact value is a double (`2^-52`). Other
    powers are themselves the subject of fits (numbers.md, "`^` / POWER are not correctly
    rounded"), so a sweep argument does not guess one."""
    if power != int(power) or abs(power) > _MAX_EXACT_EXPONENT or base == 0:
        msg = f"power {base}^{power} in an argument"
        raise UnparsedError(msg)
    value = exact(base) ** int(power)
    if round_to_bits(value, DOUBLE, DOUBLE_MIN_EXP) != value:
        msg = f"power {base}^{power} in an argument is not exact"
        raise UnparsedError(msg)
    try:
        return float(value)
    except OverflowError:
        msg = f"power {base}^{power} in an argument overflows"
        raise UnparsedError(msg) from None


# The smallest normal double: Excel returns a smaller nonzero result of `*`, `/` and `^` as 0 and
# reads a smaller literal as 0, but keeps a subnormal sum or difference (numbers.md, "Subnormal
# results become 0"; probes/subnormal-function-results, probes/precision-and-rounding A14, A70).
_SMALLEST_NORMAL = 2.0**-1022


def _flush_subnormal(value: float) -> float:
    """0 for a nonzero `value` below the smallest normal double, else `value`."""
    return 0.0 if 0 < abs(value) < _SMALLEST_NORMAL else value


class _Arithmetic:
    """A recursive-descent parser over `_tokens(text)`, evaluating as it goes (see `arithmetic`)."""

    def __init__(self, text: str, resolve: Resolve | None) -> None:
        self.text = text
        self.tokens = _tokens(text)
        self.pos = 0
        self.resolve = resolve

    def fail(self, what: str) -> UnparsedError:
        return UnparsedError(f"{what} in argument {self.text!r}")

    def peek(self) -> str | None:
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def take(self) -> str:
        if self.pos >= len(self.tokens):
            raise self.fail("unexpected end")
        self.pos += 1
        return self.tokens[self.pos - 1]

    def primary(self, depth: int) -> float:
        if depth > MAX_NESTING:
            raise self.fail("nesting too deep")
        tok = self.take()
        if tok in ("+", "-"):
            value = self.primary(depth + 1)
            return -value if tok == "-" else value
        if tok == "(":
            value = self.additive(depth + 1)
            if self.take() != ")":
                raise self.fail("unbalanced parentheses")
            return value
        if not _NUMBER.fullmatch(tok):
            if tok in _OPERATORS or self.resolve is None:
                raise self.fail(f"unexpected {tok!r}")
            return self.resolve(tok)
        literal = number_literal(tok)
        if not math.isfinite(literal):
            raise self.fail(f"non-finite literal {tok}")
        return _flush_subnormal(literal)

    def power(self, depth: int) -> float:
        value = self.primary(depth)
        while self.peek() == "^":
            self.take()
            # Excel's ^ is left-associative and flushes a subnormal power to 0
            value = _flush_subnormal(_exact_power(value, self.primary(depth)))
        return value

    def multiplicative(self, depth: int) -> float:
        value = self.power(depth)
        while (op := self.peek()) in ("*", "/"):
            self.take()
            rhs = self.power(depth)
            if op == "/" and rhs == 0:
                raise self.fail("division by zero")
            value = _flush_subnormal(X87.mul(value, rhs) if op == "*" else X87.div(value, rhs))
        return value

    def additive(self, depth: int) -> float:
        value = self.multiplicative(depth)
        while (op := self.peek()) in ("+", "-"):
            self.take()
            rhs = self.multiplicative(depth)
            value = X87.add(value, rhs) if op == "+" else X87.sub(value, rhs)
        return value


def arithmetic(text: str, resolve: Resolve | None = None) -> float:
    """Evaluates `+ - * / ^`, unary signs and parentheses over numeric literals with Excel's
    operator model: each of `+ - * /` rounds its exact result to 64 bits, then to a double
    (numbers.md, "Division rounds twice"), and `^` must be exact (`_exact_power`). A subnormal
    result of `*`, `/` or `^`, or a subnormal literal, is 0; `+` and `-` keep one. As in Excel,
    a unary sign binds tighter than `^` (`-2^2` is 4). A non-finite literal, an overflow, or
    nesting deeper than 50 is UnparsedError. A single-cell reference is an operand only when
    `resolve` is given: it maps the reference's text to the cell's number or raises UnparsedError
    (`parse_arg`'s `cell_refs`, T-3924); without it a reference is UnparsedError."""
    parser = _Arithmetic(text, resolve)
    try:
        result = parser.additive(0)
    except OverflowError:  # an X87 operation's result is beyond the double range
        raise parser.fail("overflow") from None
    if parser.pos != len(parser.tokens):
        raise parser.fail("trailing input")
    return result


def string_literal(text: str) -> str | None:
    """The text of a formula string literal (`"a""b"` is `a"b`), or None when `text` is not one."""
    if len(text) >= 2 and text[0] == text[-1] == '"' and '"' not in text[1:-1].replace('""', ""):
        return text[1:-1].replace('""', '"')
    return None


def _literal(text: str) -> float | bool | str | None:
    """A number, TRUE/FALSE or string literal, or None when `text` is none of those."""
    if _NUMBER.fullmatch(text):
        # Excel has no negative zero: `-0` is 0.
        return number_literal(text) + 0.0
    if text.upper() in ("TRUE", "FALSE"):
        return text.upper() == "TRUE"
    return string_literal(text)


def _scalar(text: str) -> float | bool | str:
    """An array item: a literal, else UnparsedError."""
    literal = _literal(text)
    if literal is None:
        msg = f"cannot evaluate literal {text!r}"
        raise UnparsedError(msg)
    return literal


def _array(text: str) -> list[float | bool | str | None]:
    """`{a,b;c,d}`: the literals in row-major order."""
    return [_scalar(item) for item in split_args(text[1:-1].replace(";", ","))]


def date_serial(year: int, month: int, day: int) -> float:
    """DATE(y, m, d) for a real calendar date from 1900-03-01 on (the serial Excel stores)."""
    try:
        when = datetime.date(year, month, day)
    except ValueError:
        msg = f"DATE({year},{month},{day}) is not a calendar date"
        raise UnparsedError(msg) from None
    if when < datetime.date(1900, 3, 1):
        msg = f"DATE({year},{month},{day}) is before 1900-03-01"
        raise UnparsedError(msg)
    return float((when - EPOCH).days)


def serial_date(serial: float) -> datetime.date:
    """The calendar date of a 1900-system serial from 61 (1900-03-01) to 2958465 (9999-12-31);
    UnparsedError for anything else (a fraction is dropped, as Excel's date functions do)."""
    if not (math.isfinite(serial) and 61 <= serial < 2958466):
        msg = f"date serial {serial} is outside 1900-03-01..9999-12-31"
        raise UnparsedError(msg)
    return EPOCH + datetime.timedelta(days=int(serial))


def parse_arg(text: str, sheet: str, book: Workbook, *, cell_refs: bool = False) -> Arg:
    """One argument's value (see the module doc for what is supported). With `cell_refs`,
    arithmetic may also read single cells of `book` that hold a number (`cell_number`):
    `A112*A112`, `1/(1+20/(4*A1249))` (T-3924, the math sidecars)."""
    if len(text) >= 2 and text[0] == "{" and text[-1] == "}":
        return _array(text)
    if d := _DATE.fullmatch(text):
        return date_serial(int(d.group(1)), int(d.group(2)), int(d.group(3)))
    if (literal := _literal(text)) is not None:
        return literal
    if m := _REF.fullmatch(text):
        name = _sheet_name(m.group("sheet"), sheet)
        (r1, c1), (r2, c2) = parse_a1(m.group("first")), parse_a1(m.group("last") or m.group("first"))
        rows, cols = range(min(r1, r2), max(r1, r2) + 1), range(min(c1, c2), max(c1, c2) + 1)
        if len(rows) * len(cols) > _MAX_RANGE_CELLS:
            msg = f"range {text} is too large"
            raise UnparsedError(msg)
        values = [book.value(name, xl_rowcol_to_cell(r, c)) for r in rows for c in cols]
        return values if m.group("last") else values[0]
    if (call := book.calls.get(call_key(text))) is not None:
        return call
    value = arithmetic(text, (lambda ref: cell_number(ref, sheet, book)) if cell_refs else None)
    # Excel has no negative zero: `-A1` over a 0 is 0, as the literal `-0` is (`_literal`)
    return value + 0.0 if cell_refs else value


def cell_number(ref: str, sheet: str, book: Workbook) -> float:
    """The number in the cell `ref` names (on `sheet` unless it names a sheet), as an arithmetic
    operand; UnparsedError for a cell holding anything else. A non-finite or subnormal number is
    UnparsedError too: whether Excel reads a stored subnormal as 0, as it reads a subnormal literal,
    has no evidence (numbers.md, "Subnormal results become 0")."""
    m = _REF.fullmatch(ref)
    if m is None or m.group("last"):
        msg = f"{ref!r} is not a single cell"
        raise UnparsedError(msg)
    row, col = parse_a1(m.group("first"))
    value = book.value(_sheet_name(m.group("sheet"), sheet), xl_rowcol_to_cell(row, col))
    if isinstance(value, bool) or not isinstance(value, float):
        msg = f"{ref} does not hold a number"
        raise UnparsedError(msg)
    if not math.isfinite(value) or _flush_subnormal(value) != value:
        msg = f"{ref} holds {value}, which is not a normal double or 0"
        raise UnparsedError(msg)
    return value


def call_key(text: str) -> str:
    """A nested call's lookup key: its text without whitespace (`PMT(0.2, 1, 10000)` and
    `PMT(0.2,1,10000)` are one call)."""
    return re.sub(r"\s+", "", text)


def call_values(case_ids: list[str]) -> dict[str, float]:
    """Excel's value of every cell of `case_ids` whose formula is one function call with a numeric
    oracle value, keyed by `call_key` of the formula without its `=`."""
    out: dict[str, float] = {}
    for case_id in case_ids:
        entry = case_entry(case_id)
        book = load_workbook(Path(entry["path"]), Path(entry["sidecar"]))
        for sheet, cells in book.sheets.items():
            for address, raw in cells.items():
                if not (isinstance(raw, str) and _CALL.fullmatch(raw)):
                    continue
                value = book.oracle.get(sheet, {}).get(address)
                if isinstance(value, dict) and value.get("t") == "n":
                    out[call_key(raw[1:])] = float(value["v"])
    return out


def case_entry(case_id: str) -> dict[str, Any]:
    """The harness's listing entry for `suite/name` (with or without `.case.json`)."""
    suite, _, name = case_id.partition("/")
    wanted = f"{suite}/{name}" if name.endswith(".case.json") else f"{suite}/{name}.case.json"
    for entry in suite_cases(suite):
        if entry["id"] == wanted:
            return entry
    msg = f"case {wanted} not found in suite {suite}"
    raise FileNotFoundError(msg)


def load_workbook(case_path: Path, sidecar: Path) -> Workbook:
    """Reads a case and its oracle sidecar (JSON; see docs/architecture/oracle.md)."""
    spec = json.loads(case_path.read_text(encoding="utf-8"))
    if not sidecar.exists():
        msg = f"{case_path.name} has no oracle sidecar yet ({sidecar}); fill it first"
        raise FileNotFoundError(msg)
    oracle = json.loads(sidecar.read_text(encoding="utf-8"))
    return Workbook(sheets={s["name"]: s["cells"] for s in spec["sheets"]}, oracle=oracle["sheets"])


def _arg_texts(m: re.Match[str]) -> list[str]:
    """One group: the argument list, split at top-level commas. Several groups: one argument per
    group that matched (so alternatives in a pattern can leave some out)."""
    groups = m.groups()
    if len(groups) <= 1:
        return split_args(groups[0]) if groups and groups[0] is not None else []
    return [g.strip() for g in groups if g is not None]


def select(
    case_id: str,
    book: Workbook,
    pattern: re.Pattern[str],
    sweep: Sweep,
    where: Any = None,
    *,
    errors: bool = False,
    complex_text: bool = False,
) -> None:
    """Adds `book`'s cells whose formula matches `pattern` to `sweep`. `where(cell)`, when given,
    keeps only the cells it returns true for. With `errors`, cells whose Excel value is an error
    are kept too (a `Cell` with `error` set), so a candidate is scored on rejecting them. With
    `complex_text`, a cell whose Excel value is an IM* function's text (`parse_complex_text`) is
    kept with its two parts (`expected`, `expected_im`)."""
    for sheet, cells in book.sheets.items():
        for address, raw in cells.items():
            if not (isinstance(raw, str) and (m := pattern.match(raw))):
                continue
            entry = book.oracle.get(sheet, {}).get(address)
            # An undecoded rich error is no evidence of which error Excel returned (T-2397).
            is_error = (
                errors and isinstance(entry, dict) and entry.get("t") == "e" and not is_undecoded(entry)
            )
            parts = _complex_parts(entry) if complex_text else None
            if not isinstance(entry, dict) or (entry.get("t") != "n" and not is_error and parts is None):
                sweep.non_numeric += 1
                continue
            try:
                args = tuple(parse_arg(a, sheet, book) for a in _arg_texts(m))
            except UnparsedError as e:
                sweep.unparsed.append(f"{case_id} {sheet}!{address}: {e}")
                continue
            if is_error:
                cell = Cell(case_id, sheet, address, raw, args, math.nan, str(entry["v"]))
            elif parts is not None:
                cell = Cell(case_id, sheet, address, raw, args, parts[0], expected_im=parts[1])
            else:
                cell = Cell(case_id, sheet, address, raw, args, float(entry["v"]))
            if where is not None and not where(cell):
                sweep.excluded += 1
                continue
            sweep.cells.append(cell)


def _complex_parts(entry: Any) -> tuple[float, float] | None:
    """The parts of a text oracle value that is a complex number, else None."""
    if not (isinstance(entry, dict) and entry.get("t") == "s"):
        return None
    try:
        return parse_complex_text(str(entry.get("v")))
    except ValueError:
        return None


class RefusedSweepError(ValueError):
    """A sweep the fit lab may not tune on (Gate A's tables, SPEC-0029 "The R4 guards")."""


def refuse_accuracy(case_ids: list[str], holdout_only: bool) -> None:
    """Gate A's reference tables (`accuracy/*`) never feed a fit: a table used to tune a constant
    would stop measuring generalization (decision 0188 R4, SPEC-0029). They may be read only to
    measure a frozen fit (`holdout_only`)."""
    tables = [c for c in case_ids if c.split("/", 1)[0] == "accuracy"]
    if tables and not holdout_only:
        msg = (
            f"refusing Gate A tables as a sweep ({', '.join(tables)}): they are a hold-out, never "
            "fit data (SPEC-0029, decision 0188 R4); pass --holdout-only to measure a frozen fit"
        )
        raise RefusedSweepError(msg)


def load_sweep(
    case_ids: list[str],
    pattern: str,
    where: Any = None,
    errors: bool = False,
    calls: list[str] | None = None,
    *,
    holdout_only: bool = False,
    complex_text: bool = False,
) -> Sweep:
    """The matching cells of every case in `case_ids` (`suite/name`); a nested call in an argument
    reads Excel's value of the same call from the `calls` cases (`call_values`). Gate A tables are
    refused unless `holdout_only` (`refuse_accuracy`)."""
    refuse_accuracy([*case_ids, *(calls or [])], holdout_only)
    regex = re.compile(pattern)
    sweep = Sweep()
    known = call_values(calls) if calls else {}
    for case_id in case_ids:
        entry = case_entry(case_id)
        book = load_workbook(Path(entry["path"]), Path(entry["sidecar"]))
        book = Workbook(book.sheets, book.oracle, known)
        select(entry["id"], book, regex, sweep, where, errors=errors, complex_text=complex_text)
    return sweep
