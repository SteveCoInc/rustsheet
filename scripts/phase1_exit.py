#!/usr/bin/env python3
"""One verdict per Phase 1 exit criterion of docs/milestones.md (decision 0187).

  scripts/phase1_exit.py [--report PATH] [--tasks DIR] [--catalog DIR] [--accuracy PATH] [--json]

Reads `eval/runs/latest/report.json` (written by `make eval`) and the task tree, and prints one
line per criterion, `met`, `not met` or `manual`, with the numbers behind it, then an overall
line. The criteria and their measures are decision 0187's:
- `run`: the run skipped no suite and was not dirty (otherwise nothing below can be trusted);
- `catalog`: eval/catalog/excel-functions.tsv and functions.yaml name the same functions, each
  with a parity scope (T-0040);
- `foundational` and `expert`: decision 0355's usage-tier gates (SPEC-0037 design 5), from the
  report's `usage_gates` plus every point of the accuracy check's JSON, which `make eval` writes
  (decision 0268), each point counted under its table's function: the tier's rates against its
  bars, met or not, and its failing functions, worst first; `n/a` when the report has no
  `usage_gates` or the JSON is absent, unreadable or older than the report;
- `gate A` and `function rate`, marked `info` and gating nothing (decision 0355 item 5): the
  accuracy check's `holds` (decision 0188 R1(a), now at decision 0355's 12-digit envelope) and Gate B
  (decision 0188 R1(b), `v15_checkpoint.gate_b`), each with the verdict it would have had;
- `owners`: `exit_owners.unowned_off` and `exit_owners.unowned` for every Phase 1 milestone list
  no cell (a cell owned only by a `defer:` task is unowned);
- `milestone M<k>`: the report's `milestone_exit` row (an owner count rides beside a met row);
  a row with pivot measures also names its suite, probe and catalog verdicts (decision 0274);
- `manual`: the prose items decision 0146 checks outside the rows.
Verdicts come from report.json and exit_owners.py; nothing is recomputed here but the tier
gates, which add the accuracy points to `usage_gates`'s counts. Exit 1 when any line is `not met`
or `n/a` (an unmeasured criterion is never met; an `info` line never counts), 2 when the report
cannot be read, else 0.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from fractions import Fraction
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import exit_owners  # noqa: E402 (the sys.path line above must run first)
import tasks  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "eval" / "catalog"
# Written by `rustsheet-eval accuracy` (accuracy::JSON_PATH), which `make eval` runs (decision 0268).
ACCURACY = ROOT / "eval" / "runs" / "accuracy.json"
MET, NOT_MET, MANUAL, NOT_MEASURED, INFO = "met", "not met", "manual", "n/a", "info"
# Decision 0355's usage tiers, in the order the lines print (report.json `usage_gates.tiers`).
TIERS = ("foundational", "expert")
# Failing functions a tier line names; the report lists every one.
TIER_SHOWN = 10
# The Phase 1 milestones whose exit criteria the Phase 1 exit requires (docs/milestones.md). A
# milestone joins once the report has its `milestone_exit` row (a missing row reads `not met`).
# M16, pivot tables, joined by decision 0274 and moved to Phase 2 by decision 0354; the report
# still computes its row.
PHASE1_MILESTONES = ("M1", "M2", "M5", "M6", "M7", "M8", "M9", "M10", "M11")
# Decision 0146: exit criteria measured outside the report's rows, with where they are written.
MANUAL_ITEMS = (
    ("M1 parity bugs and API v0", "docs/milestones.md, M1 exit criteria (decision 0146 item 1)"),
    ("M2 one walker, one matcher", "docs/milestones.md, M2 exit criteria (decision 0137)"),
)
# scripts/ is stdlib-only (no YAML reader), so the catalog is read by these line patterns; they
# follow functions.yaml's two entry shapes, and test_phase1_exit.py runs them on the real files so
# a layout change fails there (tools/tests/test_catalog.py checks the same sets through PyYAML).
# A functions.yaml entry, block style (`  - name: SUM`, `  - name: "N"`) or flow style
# (`  - { name: NOW, ..., scope: in, ... }`), and a block entry's `scope:` line.
CATALOG_NAME = re.compile(r'^  - (?:\{ )?name: "?([^",\s}]+)"?')
CATALOG_FLOW_SCOPE = re.compile(r"[{,] scope: (\w+)")
CATALOG_SCOPE = re.compile(r"^    scope: (\S+)\s*$")


@dataclass(frozen=True)
class Line:
    """One criterion's verdict and the numbers behind it."""

    criterion: str
    verdict: str
    detail: str


def run_line(report: dict[str, Any]) -> Line:
    run = report.get("run") or {}
    skipped = [str(s.get("name")) for s in run.get("skipped_suites") or []]
    dirty = bool(run.get("dirty"))
    problems = ([f"skipped suites: {', '.join(skipped)}"] if skipped else []) + (
        ["dirty tree"] if dirty else []
    )
    commit = run.get("commit", "?")
    if problems:
        return Line("run", NOT_MET, f"at {commit}: {'; '.join(problems)}")
    return Line("run", MET, f"at {commit}: every suite ran, clean tree")


def excel_function_names(path: Path) -> set[str]:
    """The `name` column of excel-functions.tsv (`#` lines are provenance, the first other line the header)."""
    rows = [ln for ln in path.read_text(encoding="utf-8").splitlines() if ln and not ln.startswith("#")]
    return {ln.split("\t", 1)[0] for ln in rows[1:]}


def catalog_scopes(path: Path) -> dict[str, str]:
    """Each functions.yaml entry's scope (`in` when the entry has no `scope:` key)."""
    scopes: dict[str, str] = {}
    current = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if m := CATALOG_NAME.match(line):
            current = m.group(1)
            flow = CATALOG_FLOW_SCOPE.search(line)
            scopes[current] = flow.group(1) if flow else "in"
        elif current and (m := CATALOG_SCOPE.match(line)):
            scopes[current] = m.group(1)
    return scopes


def catalog_line(catalog_dir: Path, report: dict[str, Any] | None = None) -> Line:
    """The catalog against Microsoft's list and, given a report, its out-of-scope set against the
    report's `out_of_scope` list (the functions the run left out)."""
    excel = excel_function_names(catalog_dir / "excel-functions.tsv")
    scopes = catalog_scopes(catalog_dir / "functions.yaml")
    out_names = {n for n, sc in scopes.items() if sc == "out"}
    reported = None if report is None else {str(o.get("name")) for o in report.get("out_of_scope") or []}
    missing = sorted(excel - set(scopes))
    extra = sorted(set(scopes) - excel)
    bad_scope = sorted(n for n, s in scopes.items() if s not in ("in", "out"))
    problems = (
        ([f"not in the catalog: {', '.join(missing)}"] if missing else [])
        + ([f"not an Excel function: {', '.join(extra)}"] if extra else [])
        + ([f"no parity scope: {', '.join(bad_scope)}"] if bad_scope else [])
        + (
            [f"report out_of_scope differs: {', '.join(sorted(reported ^ out_names))}"]
            if reported is not None and reported != out_names
            else []
        )
    )
    if problems:
        return Line("catalog", NOT_MET, "; ".join(problems))
    out = len(out_names)
    return Line("catalog", MET, f"{len(excel)} Excel functions, {len(excel) - out} in scope, {out} out")


def rate_line(report: dict[str, Any]) -> Line:
    """Gate B (decision 0188 R1(b)): `n/a` with the missing suites when the run did not measure it;
    a report from before T-3359 has no `gate_b` and reads `phase1_met`."""
    v15 = report.get("v15_checkpoint") or {}
    stale = list(v15.get("stale_exceptions") or [])
    unchecked = list(v15.get("unchecked_exceptions") or [])
    gate = v15.get("gate_b")
    detail = f"{v15.get('phase1_line') or 'no phase1_line in the report'}"
    if gate is not None and not isinstance(gate, dict):
        return Line("function rate", NOT_MEASURED, "Gate B unreadable: v15_checkpoint.gate_b is not an object")
    if isinstance(gate, dict) and gate.get("measured") is not True:
        listed = gate.get("missing")
        missing = ", ".join(str(s) for s in listed) if isinstance(listed, list) and listed else "unknown suites"
        return Line("function rate", NOT_MEASURED, f"Gate B not measured: the run did not score {missing}")
    met = (gate.get("met") if isinstance(gate, dict) else v15.get("phase1_met")) is True
    met = met and not stale and not unchecked
    if stale:
        detail += f"; {len(stale)} stale exception lines"
    if unchecked:
        detail += f"; {len(unchecked)} unchecked exception lines"
    return Line("function rate", MET if met else NOT_MET, detail)


def load_accuracy(accuracy: Path, report: Path) -> tuple[dict[str, Any] | None, str]:
    """The accuracy check's JSON, or `None` and why it measures nothing: absent, or older than the
    report (it would then describe another tree). Its shape is the caller's to check."""
    if not accuracy.exists():
        return None, f"no {accuracy.name}: run `make eval` (or `rustsheet-eval accuracy`)"
    if report.exists() and accuracy.stat().st_mtime < report.stat().st_mtime:
        return None, f"{accuracy.name} is older than the report: run `make eval`"
    try:
        top = json.loads(accuracy.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        return None, f"{accuracy.name} unreadable: {e}"
    if not isinstance(top, dict):
        return None, f"{accuracy.name} unreadable: not an object"
    return top, ""


def gate_a_line(accuracy: Path, report: Path) -> Line:
    """Gate A (decision 0188 R1(a)): the accuracy check's `holds`, with its counts. `n/a` when the
    JSON is absent, unreadable or older than the report, since it would then describe another tree."""
    top, why = load_accuracy(accuracy, report)
    if top is None:
        return Line("gate A", NOT_MEASURED, why)
    try:
        check = top["check"]
        points = list(check["points"])
        missing = len(check["missing_tables"])
        problems = sum(len(check[k]) for k in ("stale_known_failures", "bad_marks", "bad_exemptions"))
        holds = top["holds"]
    except (KeyError, TypeError) as e:
        return Line("gate A", NOT_MEASURED, f"{accuracy.name} unreadable: {e}")
    standing: dict[str, int] = {}
    for pt in points:
        key = str(pt.get("standing")) if isinstance(pt, dict) else "?"
        standing[key] = standing.get(key, 0) + 1
    inside, known = standing.get("inside", 0), standing.get("known_failure", 0)
    failing = len(points) - inside - known
    detail = (
        f"{len(points)} points, {inside} inside, {known} known failures, {failing} failing, "
        f"{problems} list problems; missing tables {missing}"
    )
    return Line("gate A", MET if holds is True else NOT_MET, detail)


def as_info(line: Line) -> Line:
    """`line` marked `info`, its own verdict kept in the detail (decision 0355 item 5)."""
    return Line(line.criterion, INFO, f"{line.verdict} (information, gates nothing): {line.detail}")


def point_class(point: object) -> str | None:
    """How an accuracy point counts for its tier: `inside`, `wrong_type` (an error or type against
    Excel's, decision 0355 item 2) or `outside`; `None` for a table defect, which is the table's,
    not the engine's. The classes are the check's own (`accuracy::envelope::Envelope`)."""
    if not isinstance(point, dict):
        return None
    cls = point.get("class")
    if cls in ("excel", "math", "both"):
        return "inside"
    if cls == "error_mismatch":
        return "wrong_type"
    if cls == "outside":
        engine = point.get("engine")
        number = isinstance(engine, dict) and engine.get("t") == "n"
        return "outside" if number else "wrong_type"
    return None


def share_meets(cells: int, misses: int, bar: object) -> bool:
    """Whether `cells - misses` of `cells` reach `bar` (a share, compared exactly); no cell and no
    miss meets, a miss without cells never does. The same rule as usage_gates.rs `Tally`."""
    if cells == 0:
        return misses == 0
    return Fraction(cells - misses) >= Fraction(str(bar)) * cells


def tier_lines(report: dict[str, Any], accuracy: dict[str, Any] | None, why: str) -> list[Line]:
    """Decision 0355's `foundational` and `expert` lines: `usage_gates` plus the accuracy points,
    each counted under its table's function, against the tier's bars (SPEC-0037 design 5)."""
    gates = report.get("usage_gates")
    tiers = gates.get("tiers") if isinstance(gates, dict) else None
    if not isinstance(gates, dict) or not isinstance(tiers, list) or not tiers:
        return [Line(t, NOT_MEASURED, "no usage_gates in the report: run `make eval`") for t in TIERS]
    if accuracy is None:
        return [Line(t, NOT_MEASURED, f"{why}; the accuracy points count in the tier gates") for t in TIERS]
    try:
        points = list(accuracy["check"]["points"])
    except (KeyError, TypeError) as e:
        return [Line(t, NOT_MEASURED, f"accuracy JSON unreadable: {e}") for t in TIERS]
    functions = [f for f in gates.get("functions") or [] if isinstance(f, dict)]
    usage = {str(f.get("name")): str(f.get("usage")) for f in functions}
    # name -> [cells, wrong type, outside], the eval's counts plus the points.
    counts = {
        str(f.get("name")): [int(f.get("cells") or 0), int(f.get("wrong_type") or 0), int(f.get("outside") or 0)]
        for f in functions
    }
    added = {t: 0 for t in TIERS}
    for pt in points:
        cls = point_class(pt)
        table = str(pt.get("table")) if isinstance(pt, dict) else ""
        if cls is None or table not in counts:
            continue
        c = counts[table]
        c[0] += 1
        c[1] += cls == "wrong_type"
        c[2] += cls == "outside"
        added[usage[table]] = added.get(usage[table], 0) + 1
    lines = []
    by_usage = {str(t.get("usage")): t for t in tiers if isinstance(t, dict)}
    for name in TIERS:
        tier = by_usage.get(name)
        if tier is None:
            lines.append(Line(name, NOT_MEASURED, f"no {name} tier in usage_gates"))
            continue
        pooled, per = tier.get("pooled_bar") or {}, tier.get("function_bar") or {}
        mine = {n: c for n, c in counts.items() if usage[n] == name}
        cells = sum(c[0] for c in mine.values())
        wrong = sum(c[1] for c in mine.values())
        outside = sum(c[2] for c in mine.values())
        type_ok = share_meets(cells, wrong, pooled.get("type", 1))
        envelope_ok = share_meets(cells, outside, pooled.get("envelope", 1))

        def meets(c: list[int]) -> bool:
            return share_meets(c[0], c[1], per.get("type", 0)) and share_meets(c[0], c[2], per.get("envelope", 0))

        failing = sorted(
            (
                (n, c)
                for n, c in mine.items()
                if not meets(c) or (not type_ok and c[1]) or (not envelope_ok and c[2])
            ),
            key=lambda item: (-(item[1][1] + item[1][2]), item[0]),
        )
        met = cells > 0 and type_ok and envelope_ok and all(meets(c) for c in mine.values())

        def pct(misses: int) -> str:
            return f"{100 * (cells - misses) / cells:.4f}%" if cells else "n/a"

        def bar(b: object) -> str:
            return f"{100 * float(str(b)):g}%"

        per_function = (
            f"; per function type ≥ {bar(per.get('type', 0))}"
            + (f", envelope ≥ {bar(per.get('envelope'))}" if float(str(per.get("envelope", 0))) else "")
        )
        detail = (
            f"{len(mine)} functions, {cells} cells ({added.get(name, 0)} accuracy points): "
            f"type {pct(wrong)} ({wrong} wrong, bar ≥ {bar(pooled.get('type', 1))}), "
            f"envelope {pct(outside)} ({outside} outside, bar ≥ {bar(pooled.get('envelope', 1))}){per_function}"
        )
        if failing:
            shown = ", ".join(f"{n} {c[1]}/{c[2]}" for n, c in failing[:TIER_SHOWN])
            more = f" and {len(failing) - TIER_SHOWN} more" if len(failing) > TIER_SHOWN else ""
            detail += f"; failing (wrong type/outside): {shown}{more}"
        lines.append(Line(name, MET if met else NOT_MET, detail))
    return lines


def owner_counts(
    report: dict[str, Any], tasks_dir: Path, milestones: list[str]
) -> tuple[int, int, dict[str, int]]:
    """(unowned `--all-off` cells, of them owned only by a deferred task, unowned exit cells per milestone)."""
    off = exit_owners.unowned_off(report, tasks_dir)
    per = {m: len(exit_owners.unowned(report, m, tasks_dir)) for m in milestones}
    return len(off), sum(1 for _, deferred in off if deferred), per


def owners_line(off: int, deferred: int, per: dict[str, int]) -> Line:
    exits = sum(per.values())
    detail = f"{off} visibly-off cells with no open owner ({deferred} owned only by a deferred task)"
    detail += f"; {exits} milestone exit cells with no owner of their milestone"
    if exits:
        detail += " (" + ", ".join(f"{m} {n}" for m, n in per.items() if n) + ")"
    return Line("owners", MET if off == 0 and exits == 0 else NOT_MET, detail)


def verdict_of(met: object) -> str:
    """A `milestone_exit` measure's verdict: null is an input the run did not measure."""
    return NOT_MEASURED if met is None else MET if met is True else NOT_MET


def pivot_detail(row: dict[str, Any]) -> str:
    """M16's measures (decision 0274): each required suite, the probe row, and the catalog
    measures of `pivot`; empty for a row without them."""
    pivot = row.get("pivot")
    if not isinstance(pivot, dict):
        return ""
    suites = [(row.get("gen_suite"), row.get("gen_met"))] + [
        (s.get("name"), s.get("met")) for s in row.get("more_suites") or [] if isinstance(s, dict)
    ]
    parts = [f"{name} {verdict_of(met)}" for name, met in suites if name]
    parts.append(f"probes {verdict_of(row.get('probes_met'))}")
    stages = ", ".join(
        f"{s.get('stage')} {s.get('evidenced')}/{int(s.get('rows') or 0) - int(s.get('web_cannot_render') or 0)}"
        for s in pivot.get("stages") or []
        if isinstance(s, dict)
    )
    floor = f" ({stages} at the floor)" if stages else ""
    parts.append(f"evidence {verdict_of(pivot.get('evidence_met'))}{floor}")
    parts.append(f"owners {verdict_of(pivot.get('owners_met'))}")
    parts.append(f"layout {verdict_of(pivot.get('layout_met'))}")
    # Plan check (b), not measured until freshness is decided without the renderer (T-4906).
    parts.append(f"corpus render {verdict_of(pivot.get('corpus_render_met'))}")
    return "; ".join(parts)


def milestone_lines(report: dict[str, Any], per: dict[str, int]) -> list[Line]:
    rows = {str(r.get("milestone")): r for r in report.get("milestone_exit") or []}
    lines = []
    for m in PHASE1_MILESTONES:
        row = rows.get(m)
        if row is None:
            lines.append(Line(f"milestone {m}", NOT_MET, "no milestone_exit row in the report"))
            continue
        met = row.get("exit") is True
        detail = "exit row met" if met else "exit row not met"
        if extra := pivot_detail(row):
            detail += f": {extra}"
        if per.get(m):
            detail += f"; {per[m]} exit cells have no {m} owner (see owners)"
        lines.append(Line(f"milestone {m}", MET if met else NOT_MET, detail))
    return lines


def verdicts(
    report: dict[str, Any],
    tasks_dir: Path,
    catalog_dir: Path,
    gate_a: Line | None = None,
    accuracy: tuple[dict[str, Any] | None, str] | None = None,
) -> list[Line]:
    rows = [str(r.get("milestone")) for r in report.get("milestone_exit") or []]
    milestones = [m for m in PHASE1_MILESTONES if m in rows]
    off, deferred, per = owner_counts(report, tasks_dir, milestones)
    acc, why = accuracy if accuracy is not None else (None, "no accuracy JSON given")
    return [
        run_line(report),
        catalog_line(catalog_dir, report),
        *tier_lines(report, acc, why),
        # Decision 0355 item 5: the old gates stay as information beside the tier gates.
        *([as_info(gate_a)] if gate_a is not None else []),
        as_info(rate_line(report)),
        owners_line(off, deferred, per),
        *milestone_lines(report, per),
        *(Line(f"manual: {item}", MANUAL, where) for item, where in MANUAL_ITEMS),
    ]


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--report", type=Path, default=exit_owners.REPORT)
    ap.add_argument("--tasks", type=Path, default=tasks.TASKS)
    ap.add_argument("--catalog", type=Path, default=CATALOG)
    ap.add_argument(
        "--accuracy", type=Path, default=ACCURACY, help="the accuracy check's JSON (its points join the tier gates)"
    )
    ap.add_argument("--json", action="store_true", help="print the lines and the overall verdict as JSON")
    args = ap.parse_args(argv)
    gate_a = gate_a_line(args.accuracy, args.report)
    accuracy = load_accuracy(args.accuracy, args.report)
    lines = exit_owners.with_report(
        args.report, lambda r: verdicts(r, args.tasks, args.catalog, gate_a, accuracy)
    )
    if lines is None:
        return 2
    failing = [ln.criterion for ln in lines if ln.verdict in (NOT_MET, NOT_MEASURED)]
    overall = NOT_MET if failing else MET
    if args.json:
        print(
            json.dumps(
                {"overall": overall, "failing": failing, "lines": [asdict(ln) for ln in lines]}, indent=2
            )
        )
    else:
        for ln in lines:
            print(f"{ln.criterion}\t{ln.verdict}\t{ln.detail}")
        tail = f": {', '.join(failing)}" if failing else " (check the manual lines by hand)"
        print(f"Phase 1 exit: {overall}{tail}")
    return 1 if failing else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
