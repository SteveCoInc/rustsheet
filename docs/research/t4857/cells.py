"""Reading the F family's Excel cells out of the eval cases (T-4857): the one reader the coverage
audit (`audit.py`) and the scorer (`score.py`) share.

A cell is Excel evidence when its case's oracle sidecar (`*.oracle.json`, excel-graph) has a value
for it. Every call of an in-scope function in its formula is read: the function, each argument's
kind (`arg_kind`) and, where every argument resolves to a number, the numbers (`numeric_args`).
Arguments resolve through literals, arithmetic over literals (`2^-1074`, `1/(1+20/(4*3))`), typed
constant cells and exact-input cells (a formula cell whose formula is such arithmetic), the inputs
`rustsheet-accuracy math` reads too (tools/src/rustsheet_tools/accuracy/eval_math.py).
"""

from __future__ import annotations

import ast
import json
import math
import re
from collections.abc import Iterator
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]

# Longest names first, so F.DIST.RT is not read as F.DIST.
FUNCTIONS = ("F.DIST.RT", "F.DIST", "FDIST", "F.INV.RT", "F.INV", "FINV", "F.TEST", "FTEST")
CALL = re.compile(
    r"(?<![A-Za-z0-9_.])(?:_xlfn\.)?(F\.DIST\.RT|F\.DIST|FDIST|F\.INV\.RT|F\.INV|FINV|F\.TEST|FTEST)\("
)
NUMBER = re.compile(r"^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$")
ARITH = re.compile(r"^[0-9eE.+\-*/^() ]+$")
REF = re.compile(r"^(?:(?:'([^']+)'|([A-Za-z_][A-Za-z0-9_.]*))!)?\$?([A-Z]{1,3})\$?(\d+)$")
RANGE = re.compile(
    r"^(?:(?:'([^']+)'|([A-Za-z_][A-Za-z0-9_.]*))!)?\$?([A-Z]{1,3})\$?(\d+):\$?([A-Z]{1,3})\$?(\d+)$"
)
ERROR_LITERAL = re.compile(r"^#(?:N/A|DIV/0!|VALUE!|REF!|NAME\?|NUM!|NULL!|SPILL!|CALC!)$")


@dataclass
class Call:
    """One in-scope call in a cell's formula."""

    function: str  # F.DIST's cdf and pdf are "F.DIST" and "F.DIST/pdf"
    args: list[str]
    kinds: list[str]
    numbers: list[float] | None  # every argument as a number, when each resolves
    top_level: bool  # the formula is this one call (optionally times an exact 2^k scale)


@dataclass
class Cell:
    """One Excel cell of the family."""

    suite: str
    case: str
    sheet: str
    a1: str
    formula: str
    oracle: dict[str, Any]
    calls: list[Call] = field(default_factory=list)
    math: str | None = None  # the math sidecar's value, if any

    @property
    def key(self) -> str:
        return f"{self.suite}/{self.case}!{self.sheet}!{self.a1}"


def split_args(text: str) -> list[str]:
    """`text`'s top-level comma-separated arguments (quotes, parentheses and braces respected)."""
    out, depth, cur, quoted = [], 0, [], False
    for ch in text:
        if quoted:
            cur.append(ch)
            if ch == '"':
                quoted = False
            continue
        if ch == '"':
            quoted = True
        elif ch in "({":
            depth += 1
        elif ch in ")}":
            depth -= 1
        elif ch == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(ch)
    out.append("".join(cur).strip())
    return out


def find_calls(formula: str) -> Iterator[tuple[str, str, int, int]]:
    """(name, argument text, start, end) of every in-scope call in `formula`."""
    for m in CALL.finditer(formula):
        depth, quoted, i = 1, False, m.end()
        while i < len(formula) and depth:
            ch = formula[i]
            if quoted:
                quoted = ch != '"'
            elif ch == '"':
                quoted = True
            elif ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
            i += 1
        if depth == 0:
            yield m.group(1), formula[m.end() : i - 1], m.start(), i


def arithmetic(text: str) -> float | None:
    """The double of arithmetic over number literals (`2^-1074`, `(1+2)/3`), else None: an AST walk over
    numbers, unary signs and + - * / ^ (no eval; an exponent beyond 2,000 in magnitude is refused, so a formula
    cannot make Python build a huge integer)."""
    t = text.strip()
    if not t or not ARITH.match(t) or "**" in t:
        return None
    try:
        value = _arith_node(ast.parse(t.replace("^", "**"), mode="eval").body)
    except (SyntaxError, ZeroDivisionError, OverflowError, ValueError):
        return None
    return value if math.isfinite(value) else None


_BINARY = {
    ast.Add: lambda a, b: a + b,
    ast.Sub: lambda a, b: a - b,
    ast.Mult: lambda a, b: a * b,
    ast.Div: lambda a, b: a / b,
}


def _arith_node(node: ast.expr) -> float:
    if isinstance(node, ast.Constant) and isinstance(node.value, int | float) and not isinstance(node.value, bool):
        return float(node.value)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub | ast.UAdd):
        v = _arith_node(node.operand)
        return -v if isinstance(node.op, ast.USub) else v
    if isinstance(node, ast.BinOp):
        a, b = _arith_node(node.left), _arith_node(node.right)
        if isinstance(node.op, ast.Pow):
            if abs(b) > 2000:
                raise ValueError("exponent out of range")
            return float(a**b)
        op = _BINARY.get(type(node.op))
        if op is not None:
            return float(op(a, b))
    raise ValueError("not arithmetic over number literals")


def number_literal(text: str) -> float | None:
    t = text.strip()
    if NUMBER.match(t):
        try:
            return float(t)
        except ValueError:
            return None
    return None


def numeric_text(s: str) -> bool:
    """Whether text reads as a number the way Excel's coercion does for plain digits (a light test:
    a decimal or exponent number, optionally signed and space-padded)."""
    return number_literal(s.strip()) is not None


class Sheets:
    """A case's cells by sheet, for resolving references."""

    def __init__(self, case: dict[str, Any]) -> None:
        self.cells = {s["name"]: s.get("cells", {}) for s in case.get("sheets", [])}

    def raw(self, sheet: str, a1: str) -> Any:
        return self.cells.get(sheet, {}).get(a1)


def _value_kind(raw: Any) -> tuple[str, float | None]:
    """The kind and number of a referenced cell's content."""
    if raw is None:
        return "ref-empty", None
    if isinstance(raw, bool):
        return "ref-bool", float(raw)
    if isinstance(raw, int | float):
        return "ref-number", float(raw)
    if isinstance(raw, dict):
        t, v = raw.get("t"), raw.get("v")
        if t == "n":
            return "ref-number", float(v)
        if t == "b":
            return "ref-bool", float(bool(v))
        if t == "e":
            return "ref-error", None
        if t == "s":
            raw = str(v)
            if raw == "":
                return "ref-empty-text", None
            return ("ref-numeric-text" if numeric_text(raw) else "ref-text"), None
        return "ref-other", None
    if isinstance(raw, str):
        if raw.startswith("="):
            body = raw[1:].strip()
            value = arithmetic(body)
            if value is not None:
                return "ref-number", value
            if body == '""':
                return "ref-empty-text", None
            if body.upper() in ("TRUE", "FALSE", "TRUE()", "FALSE()"):
                return "ref-bool", float(body.upper().startswith("TRUE"))
            if body.upper() in ("NA()",) or ERROR_LITERAL.match(body) or body in ("1/0",):
                return "ref-error", None
            return "ref-formula", None
        if raw == "":
            return "ref-empty-text", None
        return ("ref-numeric-text" if numeric_text(raw) else "ref-text"), None
    return "ref-other", None


def arg_kind(text: str, sheets: Sheets, sheet: str) -> tuple[str, float | None]:
    """An argument's kind and, when it is one number, that number."""
    t = text.strip()
    if t == "":
        return "omitted", None
    up = t.upper()
    if up in ("TRUE", "FALSE"):
        return "bool-literal", float(up == "TRUE")
    if t.startswith('"') and t.endswith('"') and len(t) >= 2:
        s = t[1:-1]
        if s == "":
            return "empty-text", None
        return ("numeric-text" if numeric_text(s) else "text"), None
    if ERROR_LITERAL.match(up):
        return "error-literal", None
    if up in ("NA()",):
        return "error-call", None
    value = number_literal(t)
    if value is not None:
        if t.lstrip("+").startswith("-") and value == 0:
            return "number-literal", -0.0
        return "number-literal", value
    value = arithmetic(t)
    if value is not None:
        return "number-arith", value
    if t.startswith("{") and t.endswith("}"):
        elems = [e for row in t[1:-1].split(";") for e in split_args(row)]
        return ("array-1" if len(elems) == 1 else "array-multi"), None
    m = REF.match(t)
    if m:
        target = m.group(1) or m.group(2) or sheet
        kind, value = _value_kind(sheets.raw(target, f"{m.group(3)}{m.group(4)}"))
        return kind, value
    m = RANGE.match(t)
    if m:
        one = m.group(3) == m.group(5) and m.group(4) == m.group(6)
        return ("range-1" if one else "range-multi"), None
    # A reference with arithmetic on it, e.g. `B5*2^-10`: resolve each reference to its number.
    refs = re.findall(r"\$?[A-Z]{1,3}\$?\d+", t)
    if refs and not re.search(r"[A-Za-z_]\(", t):
        sub = t
        for ref in set(refs):
            kind, v = _value_kind(sheets.raw(sheet, ref.replace("$", "")))
            if v is None or kind != "ref-number":
                return "formula", None
            sub = re.sub(rf"(?<![A-Z]){re.escape(ref)}(?!\d)", repr(v), sub)
        value = arithmetic(sub.replace("e+", "e").replace("E+", "E"))
        if value is not None:
            return "ref-arith", value
    return "formula", None


SCALE = re.compile(r"^(?:\*|/)2\^-?\d+$")


def read_calls(formula: str, sheets: Sheets, sheet: str) -> list[Call]:
    """Every in-scope call of a formula, with its arguments' kinds and numbers."""
    body = formula[1:] if formula.startswith("=") else formula
    out = []
    for name, argtext, start, end in find_calls(body):
        args = split_args(argtext)
        kinds, numbers = [], []
        for a in args:
            k, v = arg_kind(a, sheets, sheet)
            kinds.append(k)
            numbers.append(v)
        function = name
        if name == "F.DIST" and len(args) == 4:
            cum = numbers[3]
            if cum is not None and cum == 0:
                function = "F.DIST/pdf"
        rest = (body[:start] + body[end:]).strip()
        top = start == 0 and (rest == "" or bool(SCALE.match(rest)))
        nums = [v for v in numbers if v is not None] if all(v is not None for v in numbers) else None
        out.append(Call(function, args, kinds, nums, top))
    return out


def case_files() -> Iterator[tuple[str, Path]]:
    """(suite, case path) of every json case in eval/suites and eval/accuracy/tables."""
    for suite_dir in sorted((ROOT / "eval/suites").iterdir()):
        for p in sorted(suite_dir.rglob("*.case.json")):
            yield suite_dir.name, p
    for p in sorted((ROOT / "eval/accuracy/tables").rglob("*.case.json")):
        yield "accuracy", p


def read_cells(paths: list[tuple[str, Path]] | None = None, pending: tuple[str, ...] = ()) -> Iterator[Cell]:
    """Every oracle-backed cell of an in-scope function; a case whose id contains one of `pending`
    counts every formula cell, oracle or not (to plan a probe before its fill)."""
    for suite, path in paths if paths is not None else case_files():
        text = path.read_text(encoding="utf-8")
        if not CALL.search(text):
            continue
        oracle_path = path.with_name(path.name.replace(".case.json", ".oracle.json"))
        planned = any(p in path.as_posix() for p in pending)
        if not oracle_path.exists() and not planned:
            continue
        oracle = json.loads(oracle_path.read_text(encoding="utf-8")).get("sheets", {}) if oracle_path.exists() else {}
        math_path = path.with_name(path.name.replace(".case.json", ".math.json"))
        math_sheets = (
            json.loads(math_path.read_text(encoding="utf-8")).get("sheets", {}) if math_path.exists() else {}
        )
        case = json.loads(text)
        sheets = Sheets(case)
        case_id = path.relative_to(ROOT).as_posix().removesuffix(".case.json")
        for sheet_name, cells in sheets.cells.items():
            for a1, raw in cells.items():
                if not (isinstance(raw, str) and raw.startswith("=") and CALL.search(raw)):
                    continue
                entry = oracle.get(sheet_name, {}).get(a1) or ({"t": "pending"} if planned else None)
                if entry is None:
                    continue
                m = math_sheets.get(sheet_name, {}).get(a1)
                cell = Cell(suite, case_id, sheet_name, a1, raw, entry, math=m.get("v") if m else None)
                cell.calls = read_calls(raw, sheets, sheet_name)
                yield cell


def oracle_number(entry: dict[str, Any]) -> float | None:
    """The oracle's number, or None for an error, text or boolean."""
    if entry.get("t") == "n":
        v = entry.get("v")
        return float(v) if v is not None else None
    return None


def oracle_label(entry: dict[str, Any]) -> str:
    """'num', an error code, 'bool' or 'text'."""
    t = entry.get("t")
    if t == "n":
        return "num"
    if t == "e":
        return str(entry.get("v"))
    if t == "b":
        return "bool"
    return "text" if t == "s" else str(t)


def finite(v: float | None) -> bool:
    return v is not None and math.isfinite(v)
