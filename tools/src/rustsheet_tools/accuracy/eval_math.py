"""Math references for eval cells (SPEC-0029 "Math references for eval cells", T-3380).

`rustsheet-accuracy math --suite S` writes `<case>.math.json` (rustsheet-math/v1) next to each spec
case's oracle sidecar: for every scored cell (a key of the oracle sidecar or of the case's
`expected`) whose formula is one call `=FUNC(args)` of a function with a Gate A definition (for an
indexed definition, MINVERSE or MMULT, one element `=INDEX(FUNC(args),row,column)`; T-4025), the
50-digit math value of its arguments with its formula text `f`. A `matrix` parameter takes an array
literal or a range with its rows (`parse_rows`), or a number as a 1x1 matrix (`matrix_rows`). A call followed by an exact scale
(`=F.DIST(A1020,1,10,FALSE)*2^600`, `/2^k`, `-0`; T-3924) gets the scaled math. The scale is exact
when `k` is an integer from -1022 to 1023 (so `2^k` is itself a normal double, as Excel's `^` gives
it) and the call's math value and the scaled value both round to normal doubles or are 0: then
Excel's `*` or `/` of its call's double by `2^k` is that double times `2^k` without rounding, and
`-0` leaves it unchanged. A call whose math is subnormal is 0 in Excel (numbers.md, "Subnormal
results become 0"), so the scaled math is not Excel's operator result and the cell is listed as
skipped (`INEXACT_SCALE`), as is one whose scaled value leaves the normal range. The runner attaches a value to
the cell's result only while `f` still matches the cell's formula, and R1(c) counts a visibly-off
numeric cell inside Gate A's envelope through it.

A candidate cell gets no math value when:
- an argument reads a formula cell (its value is the engine's or Excel's, not a literal), unless that
  cell's whole formula is arithmetic over number literals (`=(83892212*2^-28+4861903*2^-54)`, an
  exact-input cell), which is read with Excel's operator model as the fit lab reads it. An argument
  may itself be arithmetic over number literals and such cells or constant cells
  (`A112*A112`, `1/(1+20/(4*A1249))`; T-3924): the fit lab's one evaluator (`arithmetic`) gives its
  double, and the math is the function at that double;
- an argument is anything else the fit lab's sweep reader leaves unparsed, text, empty or a range
  (but for an array or matrix parameter, whose elements must be numbers);
- an indexed definition is called outside INDEX (`=MINVERSE(A1)`, an array result).
  A `text` parameter (T-4084) takes a string literal or a text cell: one of its `choices`, or for a
  complex number a text `parse_complex_text` reads whose numbers have at most 15 significant digits
  (Excel cuts a longer IM* part to 15 digits, not modelled yet: engineering.md "Engineering functions
  in the gen-m9 engineering sweep (T-2258)", probes/eng-inumber-grammar-digits) and whose parts are 0
  or normal doubles (how Excel reads a subnormal part is not established), and one
  call's complex texts share one suffix (`i` or `j`).
  A nested call, or arithmetic between call results, makes the cell a composite instead
  (`composite`, T-3993, decision 0297): it gets the outermost node's math at its inner nodes'
  doubles, unless it calls a function with no definition (`no definition for ...`);
- an array or matrix parameter of a `literal_arrays` definition is not written as an array literal (AGGREGATE over
  a range, whose hidden rows the math does not model);
- the case is a 1904 workbook and the definition reads date serials in the 1900 system
  (`Definition.serials_1900`: YEARFRAC, HOUR, MINUTE, SECOND);
- an integer parameter is given a non-integer (the definitions take integers; whether Excel
  truncates is per function, so the cell is skipped rather than guessed) unless the parameter cites
  the entry showing Excel truncates it (`Param.truncates`: then the truncated integer is the
  argument, T-3923), a logical parameter is
  given a number (likewise unevidenced per function), or the call has a
  different number of arguments than the definition (an omitted optional argument the definition
  does not mark `optional`, or an extra one);
- the generator would drop the point (dps 60 and 80 disagree at the definition's deterministic
  budget, or a value beyond the double range; never the clock, T-3680). Only these skips need mpmath, so the sidecar lists them under `skipped`
  (sheet → A1 → `f` and `why`) and `math --check` (T-3588) can tell, without mpmath, a cell the
  generator dropped from one nobody ran the generator for.
xlsx cases are skipped: tools never parse xlsx values.
"""

from __future__ import annotations

import json
import math
import multiprocessing
import re
from collections.abc import Iterable
from concurrent.futures import ProcessPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import mpmath

from rustsheet_tools.accuracy import composite, defs, literal
from rustsheet_tools.accuracy.model import REGISTRY, Definition, NoValueError, Param, Unfit
from rustsheet_tools.accuracy.tables import (
    DIGITS,
    DPS,
    MATH_FORMAT,
    DroppedError,
    MathValue,
    compute,
    math_sidecar_bytes,
)
from rustsheet_tools.fitlab.sweep import (
    UnparsedError,
    Workbook,
    arithmetic,
    parse_arg,
    parse_complex_text,
    parse_matrix_arg,
    split_args,
)
from rustsheet_tools.snapshot import suite_cases

_ = defs  # importing the package registers the definitions

NO_VALUE = "no single value"
NOT_CONVERGED = "not converged"
INEXACT_SCALE = "scaled value not exact"
REGENERATE = "cd tools && uv run rustsheet-accuracy math --suite {suite}"

_CALL = re.compile(r"=\s*([A-Za-z][A-Za-z0-9._]*)\((.*)\)\s*", re.DOTALL)
# A call's exact outer scale: `*2^k`, `/2^k` or `-0` after its closing parenthesis.
_SCALE = re.compile(r"(?P<call>.*\))\s*(?:(?P<op>[*/])\s*2\s*\^\s*(?P<k>[+-]?\d{1,4})|-\s*0)\s*", re.DOTALL)
# 2^k is a normal double for these k, so Excel's `^` gives it exactly.
_SCALE_POWERS = range(-1022, 1024)
_SMALLEST_NORMAL = 2.0**-1022
# An unsigned number of a complex text (its mantissa and exponent), for the 15-digit check.
_TEXT_NUMBER = re.compile(r"(?P<mantissa>\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?")


@dataclass(frozen=True)
class LiteralBook(Workbook):
    """A case's cells without any engine or Excel value: a referenced formula cell is read only
    when its formula is arithmetic over number literals (see the module doc)."""

    # Each formula cell's arithmetic value or why it has none: probe cases refer to the same
    # exact-input cells from many calls, and parsing them once keeps `math --check` cheap (T-3588).
    formulas: dict[str, float | UnparsedError] = field(default_factory=dict, compare=False)

    def value(self, sheet: str, address: str) -> float | bool | str | None:
        """A constant, or the value of an arithmetic-over-literals formula; UnparsedError for any
        other formula cell."""
        raw = self.sheets.get(sheet, {}).get(address)
        if isinstance(raw, str) and raw.startswith("="):
            if raw not in self.formulas:
                try:
                    self.formulas[raw] = arithmetic(raw[1:])
                except UnparsedError as err:
                    self.formulas[raw] = err
            found = self.formulas[raw]
            if isinstance(found, UnparsedError):
                raise found
            return found
        return super().value(sheet, address)


@dataclass
class CaseMath:
    """One case's math cells and why the other candidate cells got none."""

    case: str
    cells: dict[str, dict[str, dict[str, str]]] = field(default_factory=dict)  # sheet → A1 → cell
    skipped: dict[str, int] = field(default_factory=dict)  # reason → count
    # The candidates the generator dropped: sheet → A1 → {"f", "why"}, written to the sidecar.
    dropped: dict[str, dict[str, dict[str, str]]] = field(default_factory=dict)

    def skip(self, reason: str) -> None:
        """Counts one candidate cell left without a math value."""
        self.skipped[reason] = self.skipped.get(reason, 0) + 1

    def drop(self, sheet: str, a1: str, text: str, reason: str) -> None:
        """Counts a candidate the generator dropped and lists it for the sidecar's `skipped`."""
        self.skip(reason)
        self.dropped.setdefault(sheet, {})[a1] = {"f": text, "why": reason}

    @property
    def written(self) -> int:
        """The number of cells with a math value or a math domain error."""
        return sum(len(c) for c in self.cells.values())


def math_path(oracle_sidecar: Path) -> Path:
    """`foo.oracle.json` → `foo.math.json`, beside it. Keep in step with the runner's
    `accuracy::math::math_sidecar` (crates/rustsheet-eval/src/accuracy/math.rs), which reads it."""
    name = oracle_sidecar.name.removesuffix(".oracle.json")
    return oracle_sidecar.with_name(f"{name}.math.json")


def single_call(text: str) -> tuple[str, str] | None:
    """(function name upper-cased, argument text) when `text` is one call `=FUNC(args)` whose
    opening parenthesis closes at the end, else None."""
    m = _CALL.fullmatch(text)
    if m is None:
        return None
    inner = m.group(2)
    depth, quoted = 0, False
    for ch in inner:
        if ch == '"':
            quoted = not quoted
        elif quoted:
            continue
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth < 0:  # `=F(a)+G(b)`: the first call closes before the end
                return None
    return (m.group(1).upper(), inner) if depth == 0 and not quoted else None


def scaled_call(text: str) -> tuple[str, str, int | None] | None:
    """(function name, argument text, scale) when `text` is one call (`single_call`, scale None)
    or one call followed by an exact scale (see the module doc): `*2^k` is `k`, `/2^k` is `-k`
    and `-0` is 0. None for any other formula."""
    if (call := single_call(text)) is not None:
        return (*call, None)
    m = _SCALE.fullmatch(text)
    if m is None or (call := single_call(m.group("call"))) is None:
        return None
    power = 0 if m.group("op") is None else int(m.group("k")) * (1 if m.group("op") == "*" else -1)
    return (*call, power) if power in _SCALE_POWERS else None


def scale_math(value: MathValue, power: int) -> MathValue | None:
    """The math of a call's `value` times `2^power`, or None when the scale is not exact (the call's
    or the scaled value rounds to a nonzero double below the normal range, or the scaled value is
    beyond it). A domain error is the same error scaled (Excel's `#NUM!*2^600` is `#NUM!`)."""
    if value.value is None:
        return value
    with mpmath.workdps(DPS[-1]):
        unscaled = mpmath.mpf(value.value)
        scaled = mpmath.ldexp(unscaled, power)
        if not (_normal_or_zero(unscaled) and _normal_or_zero(scaled)):
            return None
        # `-0` and a 0 keep the generator's own text; any other scale is written to DIGITS again
        return value if power == 0 or unscaled == 0 else MathValue(value=mpmath.nstr(scaled, DIGITS))


def _normal_or_zero(value: mpmath.mpf) -> bool:
    """Whether `value` rounds to a normal double or is 0."""
    nearest = abs(float(value))
    return value == 0 or (nearest >= _SMALLEST_NORMAL and math.isfinite(nearest))


def array_elements(value: Any) -> tuple[float, ...] | str:
    """A range or array argument's numbers in row-major order, or why it does not fit (a scalar, or
    an element that is not a number: the definitions take dense numeric arrays only)."""
    if not isinstance(value, list):
        return "scalar for an array"
    flat: list[Any] = []
    for row in value:
        flat.extend(row if isinstance(row, list) else [row])
    if not flat or any(isinstance(e, (bool, str)) or e is None for e in flat):
        return "non-numeric array element"
    return tuple(float(e) for e in flat)


def fit_arg(param: Param, value: Any) -> literal.Arg | Unfit:
    """One argument as the parameter's kind, or why it does not fit (an `Unfit`: a text argument
    is a plain str)."""
    if param.kind == "text":
        return fit_text(param, value)
    fitted: literal.Arg | str
    if param.kind == "array":
        fitted = array_elements(value)
    elif param.kind == "matrix":
        fitted = matrix_rows(value)
    else:
        fitted = _fit_number(param, value)
    return Unfit(fitted) if isinstance(fitted, str) else fitted


def matrix_rows(value: Any) -> literal.Matrix | str:
    """A matrix argument (`parse_matrix_arg`: rows of values, or a scalar) as the definitions take
    it, or why it does not fit. A number is a 1x1 matrix: `MDETERM(5)` is 5 (probes/stats-evidence-floor-t1623
    S!Q112), and MDETERM of a single numeric cell is that number (arrays.md "MDETERM / MINVERSE: a blank
    cell in the matrix is #VALUE!"). A blank, a logical or a text, as the matrix or one of its elements, is
    `#VALUE!` in Excel (same entry), which the math does not model; a flat list (a range `parse_arg` read
    without its shape) and ragged rows are refused too."""
    if isinstance(value, float):
        return ((value,),)
    if not isinstance(value, list) or not value or not all(isinstance(row, list) for row in value):
        return "a matrix argument without its shape" if isinstance(value, list) else "non-numeric argument"
    if len({len(row) for row in value}) != 1 or not value[0]:
        return "ragged matrix rows"
    if any(isinstance(e, (bool, str)) or e is None for row in value for e in row):
        return "non-numeric matrix element"
    return tuple(tuple(float(e) for e in row) for row in value)


def _fit_number(param: Param, value: Any) -> literal.Arg | str:
    """`fit_arg` for a scalar kind (real, int, bool): the argument, or why it does not fit (a str)."""
    if isinstance(value, list) or value is None or isinstance(value, str):
        return "non-numeric argument"
    if param.kind == "bool":
        # no evidence yet of how each function reads a number here
        return value if isinstance(value, bool) else "number for a logical"
    if isinstance(value, bool):
        return "logical for a number"
    if param.kind == "int":
        return fit_int(param, float(value))
    return float(value)


def fit_text(param: Param, value: Any) -> str | Unfit:
    """A text for a `text` parameter as is, or why it does not fit: a non-text (whether each
    function reads a number as its text is not evidenced), a text outside `choices`, or for a complex
    number one `parse_complex_text` does not read, with a number past 15 significant digits (Excel cuts
    it to 15 digits, not modelled yet: engineering.md "Engineering functions in the gen-m9 engineering
    sweep (T-2258)", probes/eng-inumber-grammar-digits), or with a subnormal part (how Excel reads one is
    not established)."""
    if not isinstance(value, str):
        return Unfit("non-text for a text parameter")
    if param.choices:
        return value if value in param.choices else Unfit("text outside the parameter's list")
    try:
        parts = parse_complex_text(value)
    except ValueError:
        return Unfit("not a complex number text")
    for m in _TEXT_NUMBER.finditer(value):
        if len(m.group("mantissa").replace(".", "").strip("0")) > 15:
            return Unfit("complex number part past 15 significant digits")
    if not all(_normal_or_zero(mpmath.mpf(part)) for part in parts):
        return Unfit("subnormal complex number part")
    return value


def fit_int(param: Param, value: float) -> int | str:
    """A number for an `int` parameter: an integer as is, and a non-integer truncated toward zero
    where `param.truncates` cites the evidence that Excel does so, or why it does not fit."""
    if not math.isfinite(value):
        return "non-finite for an integer parameter"
    if value.is_integer():
        return int(value)
    return math.trunc(value) if param.truncates else "non-integer for an integer parameter"


def fit_args(definition: Definition, values: list[Any]) -> list[literal.Arg] | str:
    """`values` as the definition's arguments, or why they do not fit."""
    try:
        params = definition.params_for(len(values))
    except ValueError:
        return "argument count"
    out: list[literal.Arg] = []
    suffixes: set[str] = set()
    for param, value in zip(params, values, strict=True):
        arg = fit_arg(param, value)
        if isinstance(arg, Unfit):
            return str(arg)
        if param.is_complex and isinstance(arg, str) and arg[-1:] in literal.COMPLEX_SUFFIXES:
            suffixes.add(arg[-1])
        out.append(arg)
    # mixed suffixes are an error (IMSUM's #VALUE!, IMSUB's and IMDIV's #NUM!; engineering.md "IMSUM and
    # IMPRODUCT take their arguments whole"), which the math (an `mpc` per text) does not model
    return "mixed complex suffixes" if len(suffixes) > 1 else out


def scored_cells(spec: dict[str, Any], oracle: dict[str, Any] | None) -> Iterable[tuple[str, str]]:
    """(sheet, A1) of every scored cell: the oracle sidecar's cells and the case's `expected`."""
    seen: set[tuple[str, str]] = set()
    for sheet, cells in ((oracle or {}).get("sheets") or {}).items():
        for a1 in cells:
            seen.add((sheet, a1.upper()))
    for key in spec.get("expected") or {}:
        sheet, sep, a1 = key.rpartition("!")
        if sep:
            seen.add((sheet, a1.upper()))
    return sorted(seen)


@dataclass(frozen=True)
class Candidate:
    """A scored one-call cell of a function with a definition: its arguments as the definition
    reads them, or why it gets no math value without running the generator. A composite cell
    (`composite`, T-3993) has no `definition`; its `tree` is the parsed formula."""

    sheet: str
    a1: str
    text: str
    definition: Definition | None
    args: list[literal.Arg] | str
    scale: int | None = None  # the exact outer scale's power of two (`scaled_call`), else None
    tree: composite.Node | None = None

    def compute(self) -> MathValue:
        """The math value of a candidate whose arguments fit (`args` is not a reason)."""
        if self.tree is not None:
            return composite.compute(self.tree)
        if self.definition is None or isinstance(self.args, str):
            msg = f"{self.sheet}!{self.a1} has no math to compute"
            raise ValueError(msg)
        return compute(self.definition, self.args)


def candidates(case_path: Path, sidecar: Path) -> list[Candidate]:
    """The candidate cells of one spec case, in sheet and A1 order (no mpmath runs here)."""
    spec = json.loads(case_path.read_text(encoding="utf-8"))
    oracle = json.loads(sidecar.read_text(encoding="utf-8")) if sidecar.exists() else None
    sheets = {s["name"]: s["cells"] for s in spec["sheets"]}
    book = LiteralBook(sheets=sheets, oracle={})
    out: list[Candidate] = []
    for sheet, a1 in scored_cells(spec, oracle):
        text = sheets.get(sheet, {}).get(a1)
        if not isinstance(text, str):
            continue
        if (call := scaled_call(text)) is None:
            if (tree := composite.parse(text, sheet, book, fit_arg)) is not None:
                out.append(Candidate(sheet, a1, text, None, tree.why or [], tree=tree.root))
            continue
        name, inner, scale = call
        name, texts = indexed_call(name, inner)
        definition = REGISTRY.get(name)
        if definition is None:
            continue
        if definition.serials_1900 and spec.get("date_system") == 1904:
            out.append(Candidate(sheet, a1, text, definition, "a 1904 workbook", scale))
            continue
        if definition.indexed and texts is None:
            out.append(Candidate(sheet, a1, text, definition, "an array result outside INDEX", scale))
            continue
        texts = split_args(inner) if texts is None else texts
        if definition.literal_arrays and _range_for_literal(definition, texts):
            out.append(Candidate(sheet, a1, text, definition, "not an array literal", scale))
            continue
        try:
            values = _parse_values(definition, texts, sheet, book)
        except (UnparsedError, ValueError) as err:
            if (tree := composite.parse(text, sheet, book, fit_arg)) is not None:  # a nested call
                out.append(Candidate(sheet, a1, text, None, tree.why or [], tree=tree.root))
            else:
                out.append(Candidate(sheet, a1, text, definition, f"unparsed ({type(err).__name__})", scale))
            continue
        out.append(Candidate(sheet, a1, text, definition, fit_args(definition, values), scale))
    return out


def indexed_call(name: str, inner: str) -> tuple[str, list[str] | None]:
    """For `INDEX(FUNC(args),row,column)` of an indexed definition (`Definition.indexed`, T-4025), FUNC and
    its arguments followed by the row and the column, as the definition takes them; else `name` and None."""
    outer = split_args(inner) if name == "INDEX" else []
    if len(outer) == 3 and (call := single_call(f"={outer[0]}")) is not None:
        definition = REGISTRY.get(call[0])
        if definition is not None and definition.indexed:
            return call[0], [*split_args(call[1]), *outer[1:]]
    return name, None


def _parse_values(definition: Definition, texts: list[str], sheet: str, book: Workbook) -> list[Any]:
    """The argument texts' values: a matrix parameter's row by row (`parse_matrix_arg`), any other as
    `parse_arg` reads it."""
    try:
        kinds = [p.kind for p in definition.params_for(len(texts))]
    except ValueError:
        kinds = ["real"] * len(texts)  # fit_args reports the argument count
    return [
        parse_matrix_arg(t, sheet, book) if k == "matrix" else parse_arg(t, sheet, book, cell_refs=True)
        for t, k in zip(texts, kinds, strict=True)
    ]


def _range_for_literal(definition: Definition, texts: list[str]) -> bool:
    """Whether an array or matrix parameter's argument text is anything but an array literal `{...}`."""
    try:
        params = definition.params_for(len(texts))
    except ValueError:
        return False  # fit_args reports the argument count
    return any(
        p.kind in ("array", "matrix") and not t.strip().startswith("{")
        for p, t in zip(params, texts, strict=True)
    )


def case_math(case_id: str, case_path: Path, sidecar: Path) -> CaseMath:
    """The math cells of one spec case (see the module doc for what is skipped)."""
    out = CaseMath(case_id)
    for c in candidates(case_path, sidecar):
        if isinstance(c.args, str):
            out.skip(c.args)
            continue
        try:
            value = c.compute()
        except composite.CompositeSkipError as err:
            out.drop(c.sheet, c.a1, c.text, str(err))
            continue
        except NoValueError:
            out.drop(c.sheet, c.a1, c.text, NO_VALUE)
            continue
        except DroppedError:
            out.drop(c.sheet, c.a1, c.text, NOT_CONVERGED)
            continue
        if c.scale is not None:
            scaled = scale_math(value, c.scale)
            if scaled is None:
                out.drop(c.sheet, c.a1, c.text, INEXACT_SCALE)
                continue
            value = scaled
        out.cells.setdefault(c.sheet, {})[c.a1] = {**value.to_json(), "f": c.text}
    return out


def sidecar_bytes(case_path: Path, math: CaseMath) -> bytes:
    """The math sidecar for `math`, keyed to the case file's hash."""
    sheets = {s: dict(sorted(c.items())) for s, c in sorted(math.cells.items())}
    dropped = {s: dict(sorted(c.items())) for s, c in sorted(math.dropped.items())}
    return math_sidecar_bytes(case_path.read_bytes(), sheets, skipped=dropped)


def _pool(workers: int | None = None) -> ProcessPoolExecutor:
    """The process pool `write_suite` and `check_suite` share. `spawn`, not the platform default:
    forking a multi-threaded parent (a pytest-xdist worker, decision 0233) can deadlock the child,
    and `spawn` exists on every platform (Windows too)."""
    return ProcessPoolExecutor(max_workers=workers, mp_context=multiprocessing.get_context("spawn"))


def _job(entry: dict[str, Any]) -> tuple[dict[str, Any], CaseMath]:
    return entry, case_math(entry["id"], Path(entry["path"]), Path(entry["sidecar"]))


def calls_any(case_path: Path, functions: Iterable[str]) -> bool:
    """Whether the case file's text calls one of `functions` (`NAME(`, not a longer name ending in it)."""
    text = case_path.read_text(encoding="utf-8").upper()
    return any(re.search(r"(?<![A-Z0-9._])" + re.escape(name.upper()) + r"\(", text) for name in functions)


def write_suite(
    suite: str, workers: int | None = None, functions: Iterable[str] | None = None
) -> list[CaseMath]:
    """Writes the math sidecar of every spec case of `suite` that has a math cell, and removes a
    math sidecar this tool wrote for a case that no longer has one. Deterministic: the same case
    files give the same bytes. With `functions`, only the cases that call one of them are
    recomputed and rewritten (T-4483): a new definition's cells get their values without touching
    unrelated sidecars, whose time-limited cells can flip between a value and a skip by machine."""
    entries = _spec_entries(suite)
    if functions is not None:
        names = list(functions)
        entries = [e for e in entries if calls_any(Path(e["path"]), names)]
    results: list[CaseMath] = []
    with _pool(workers) as pool:
        for entry, math in pool.map(_job, entries):
            path = math_path(Path(entry["sidecar"]))
            if math.written or math.dropped:
                path.write_bytes(sidecar_bytes(Path(entry["path"]), math))
            elif path.exists() and _ours(path):
                path.unlink()
            results.append(math)
    return results


def _spec_entries(suite: str) -> list[dict[str, Any]]:
    """The harness's listing of `suite`'s spec cases (tools never parse xlsx values)."""
    return [e for e in suite_cases(suite) if str(e["path"]).endswith(".case.json")]


def check_case(case_id: str, case_path: Path, sidecar: Path) -> list[str]:
    """Why the case's math sidecar is not what `math` would write for its cells, without running
    mpmath: a candidate cell the sidecar neither holds nor lists as skipped by the generator, or a
    sidecar entry whose `f` is not the cell's formula (or whose cell is no candidate any more)."""
    path = math_path(sidecar)
    data: dict[str, Any] = {}
    if path.exists():
        try:
            loaded = json.loads(path.read_text(encoding="utf-8"))
        except ValueError:
            return [f"{case_id}: math sidecar {path.name} is not JSON"]
        data = loaded if isinstance(loaded, dict) else {}
    held: dict[tuple[str, str], str | None] = {}
    for key in ("sheets", "skipped"):
        for sheet, cells in (data.get(key) or {}).items():
            for a1, cell in cells.items():
                held[(sheet, a1)] = cell.get("f") if isinstance(cell, dict) else None
    problems: list[str] = []
    wanted = {(c.sheet, c.a1): c.text for c in candidates(case_path, sidecar) if not isinstance(c.args, str)}
    for (sheet, a1), text in sorted(wanted.items()):
        if (sheet, a1) not in held:
            problems.append(f"{case_id}: {sheet}!{a1} {text} has no math value and is not listed as skipped")
    for (sheet, a1), f in sorted(held.items()):
        if wanted.get((sheet, a1)) != f:
            problems.append(
                f"{case_id}: {sheet}!{a1} math sidecar entry {f} is stale (the cell is {wanted.get((sheet, a1))})"
            )
    return problems


def _check_job(entry: dict[str, Any]) -> list[str]:
    """`check_case` for one listing entry, in a worker process."""
    return check_case(entry["id"], Path(entry["path"]), Path(entry["sidecar"]))


def check_suite(suite: str) -> list[str]:
    """`check_case` over every spec case of `suite`; each problem names its case, and the last
    line the command that regenerates the sidecars."""
    problems: list[str] = []
    with _pool() as pool:  # parsing every argument is the cost: one case per process
        for found in pool.map(_check_job, _spec_entries(suite)):
            problems += found
    if problems:
        problems.append(f"regenerate: {REGENERATE.format(suite=suite)}")
    return problems


def _ours(path: Path) -> bool:
    """Whether `path` is a math sidecar this tool wrote (never delete anything else)."""
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return False
    source = data.get("source") if isinstance(data, dict) else None
    return (
        isinstance(source, dict)
        and data.get("format") == MATH_FORMAT
        and source.get("generator") == "rustsheet-accuracy"
    )
