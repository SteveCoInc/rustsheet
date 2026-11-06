#!/usr/bin/env python3
"""Decision 0267 item 6's net-improvement check for incomplete-beta family regressions (T-4159).

Item 6 lets a PR record family regressions with `rustsheet-eval update-baseline --allow-regressions`
when (a) every regressed cell is a number of a family formula that lands in `residue13` or better,
(b) nothing else regresses, and (c) the family's cells within 15 digits rise against main while its
visibly-off numbers do not. This script checks (a) to (c) from two report.json files, main's and the
branch's, over the same suites, and prints the counts (d) asks the PR body to cite.

Nothing here re-derives a closeness bucket or family membership: `rustsheet-eval run` writes each
listed cell's `closeness` bucket and `thirteen_digit_family` flag (`closeness::serialize_cells`) and
the family list (`thirteen_digit_family`), and (c) sums the report's own `functions[].closeness`.
report.json lists only a case's non-passing cells, so a cell it does not list is bit-exact (or a
pinned deviation): the `pass` bucket. Each case's `checked - passed - deviations` must equal its
listed scored cells, which is what makes that reading safe.

A cell regresses when its bucket gets worse (the order of `RANK`), as in the baseline gate a newly
failing cell does. Cases whose checked-cell count changed, and cases only one report has, are new
coverage and are not compared. A text check failing on the branch and not on main is a regression:
report.json does not count a case's passing text checks, so new text coverage counts too.

Usage: family_net.py MAIN_REPORT BRANCH_REPORT
Exit 0 when (a) to (c) hold, 1 when one fails, 2 when the reports cannot be compared (unreadable,
written before T-4159, different suites or family lists, or a case whose counts do not add up).
Standard library only.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# The closeness buckets of `closeness::Closeness`, best first; an unsupported cell has none and
# ranks last. A move to a later bucket is a regression.
RANK = {
    "pass": 0,
    "ulp1": 1,
    "ulp4": 2,
    "digits15": 3,
    "residue13": 4,
    "visible_numeric": 5,
    "visible_other": 6,
}
UNSUPPORTED = len(RANK)
# The last bucket item 6 (a) allows a regressed family number to land in.
RESIDUE13 = RANK["residue13"]
# `functions[].closeness` buckets item 6 (c) counts as within 15 digits.
WITHIN15 = ("pass", "ulp1", "ulp4", "digits15")
# `CaseStatus::did_not_run`.
DID_NOT_RUN = {"engine_error", "panic", "timeout"}

Report = dict[str, Any]


class ReportError(Exception):
    """The two reports cannot be compared."""


@dataclass(frozen=True)
class Regression:
    """One cell or text check that is worse on the branch than on main."""

    cell: str  # `case!Sheet!A1`, as `runner::cell_id` names it
    before: str
    after: str
    formula: str
    allowed: bool  # item 6 (a): a family number that stays in residue13 or better


@dataclass
class Outcome:
    """What the check found; `render` prints it."""

    within15: tuple[int, int] = (0, 0)  # (main, branch)
    visible_numeric: tuple[int, int] = (0, 0)
    residue13: tuple[int, int] = (0, 0)
    compared_cases: int = 0
    uncompared_cases: int = 0
    regressions: list[Regression] = field(default_factory=list)

    @property
    def rejected(self) -> list[Regression]:
        return [r for r in self.regressions if not r.allowed]

    @property
    def net_gain(self) -> bool:
        """Item 6 (c)."""
        return self.within15[1] > self.within15[0] and self.visible_numeric[1] <= self.visible_numeric[0]

    @property
    def ok(self) -> bool:
        return not self.rejected and self.net_gain


def load(path: Path) -> Report:
    try:
        report = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as e:
        raise ReportError(f"cannot read {path}: {e}") from e
    if not isinstance(report, dict):
        raise ReportError(f"{path} is not a report.json object")
    return report


def family(main: Report, branch: Report) -> set[str]:
    names = [r.get("thirteen_digit_family") for r in (main, branch)]
    if not names[0] or not names[1]:
        raise ReportError("a report has no `thirteen_digit_family` list: re-run rustsheet-eval (T-4159)")
    if names[0] != names[1]:
        raise ReportError(f"the family lists differ: main {names[0]}, branch {names[1]}")
    return set(names[1])


def family_counts(report: Report, members: set[str]) -> tuple[int, int, int]:
    """(within 15 digits, visible_numeric, residue13) summed over the family's `functions[].closeness`."""
    within = visible = residue = 0
    for f in report.get("functions", []):
        if f.get("name") in members:
            c = f.get("closeness", {})
            within += sum(int(c.get(k, 0)) for k in WITHIN15)
            visible += int(c.get("visible_numeric", 0))
            residue += int(c.get("residue13", 0))
    return within, visible, residue


def rank(cell: dict[str, Any] | None, case: str) -> tuple[int, str]:
    """`cell`'s bucket rank and name; an unlisted cell (None) passes."""
    if cell is None:
        return 0, "pass"
    bucket = cell.get("closeness")
    if bucket is None:
        if cell.get("verdict") == "unsupported":
            return UNSUPPORTED, "unsupported"
        raise ReportError(f"{case}: a listed cell has no `closeness`: re-run rustsheet-eval (T-4159)")
    if bucket not in RANK:
        raise ReportError(f"{case}: unknown closeness bucket {bucket!r}")
    return RANK[bucket], bucket


def listed(case: Report) -> dict[tuple[str, str, str], dict[str, Any]]:
    """The case's listed cells by (sheet, cell, kind), after checking they account for every
    checked cell that is not bit-exact."""
    cells = {(c["sheet"], c["cell"], c.get("kind", "scored")): c for c in case.get("cells", [])}
    scored = sum(1 for k in cells if k[2] == "scored")
    if case.get("checked", 0) - case.get("passed", 0) - case.get("deviations", 0) != scored:
        raise ReportError(f"{case.get('id')}: checked - passed - deviations is not its {scored} listed cells")
    return cells


def compare_case(case_id: str, main: Report, branch: Report) -> list[Regression]:
    out: list[Regression] = []
    if branch.get("status") in DID_NOT_RUN and main.get("status") not in DID_NOT_RUN:
        out.append(Regression(case_id, str(main.get("status")), str(branch.get("status")), "", allowed=False))
        return out
    before, after = listed(main), listed(branch)
    for key in sorted(before.keys() | after.keys()):
        old, new = rank(before.get(key), case_id), rank(after.get(key), case_id)
        if new[0] <= old[0]:
            continue
        cell = after.get(key) or {}
        out.append(
            Regression(
                f"{case_id}!{key[0]}!{key[1]}",
                old[1],
                new[1],
                str(cell.get("formula") or cell.get("anchor_formula") or ""),
                allowed=bool(cell.get("thirteen_digit_family")) and new[0] <= RESIDUE13,
            )
        )
    failing_text = {(t["sheet"], t["cell"]) for t in main.get("text", []) if not t.get("pass")}
    for t in branch.get("text", []):
        if not t.get("pass") and (t["sheet"], t["cell"]) not in failing_text:
            out.append(Regression(f"{case_id}!{t['sheet']}!{t['cell']}", "text pass", "text fail", "", False))
    return out


def check(main: Report, branch: Report) -> Outcome:
    suites = [sorted(s.get("name", "") for s in r.get("suites", [])) for r in (main, branch)]
    if suites[0] != suites[1]:
        raise ReportError(f"the reports ran different suites: main {suites[0]}, branch {suites[1]}")
    members = family(main, branch)
    before, after = family_counts(main, members), family_counts(branch, members)
    outcome = Outcome(
        within15=(before[0], after[0]), visible_numeric=(before[1], after[1]), residue13=(before[2], after[2])
    )
    main_cases = {(c.get("suite"), c.get("id")): c for c in main.get("cases", [])}
    for case in branch.get("cases", []):
        old = main_cases.get((case.get("suite"), case.get("id")))
        if old is None or old.get("checked") != case.get("checked"):
            outcome.uncompared_cases += 1
            continue
        outcome.compared_cases += 1
        outcome.regressions += compare_case(str(case.get("id")), old, case)
    return outcome


def render(o: Outcome) -> list[str]:
    def delta(pair: tuple[int, int]) -> str:
        return f"{pair[0]} -> {pair[1]} ({pair[1] - pair[0]:+d})"

    rejected = o.rejected
    lines = [
        "Decision 0267 item 6 (family: functions[].closeness of thirteen_digit_family, main -> branch)",
        f"  family within 15 digits (pass+ulp1+ulp4+digits15): {delta(o.within15)}",
        f"  family visible_numeric: {delta(o.visible_numeric)}",
        f"  family residue13: {delta(o.residue13)}",
        f"  cases compared: {o.compared_cases}; new or changed cases not compared: {o.uncompared_cases}",
        f"  regressed cells: {len(o.regressions)} ({len(o.regressions) - len(rejected)} family numbers "
        f"within residue13, {len(rejected)} other)",
        *(
            f"    {r.cell}\t{r.before} -> {r.after}\t{r.formula}" + ("" if r.allowed else "\tNOT ALLOWED")
            for r in o.regressions
        ),
        f"(a, b) {'ok' if not rejected else 'FAIL'}: "
        + (
            "every regression is a family number within residue13"
            if not rejected
            else f"{len(rejected)} regressions are not family numbers within residue13"
        ),
        f"(c) {'ok' if o.net_gain else 'FAIL'}: within 15 digits must rise and visible_numeric must not",
    ]
    return lines


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else None)
    parser.add_argument("main_report", type=Path, help="report.json of main")
    parser.add_argument("branch_report", type=Path, help="report.json of the branch")
    args = parser.parse_args(argv)
    try:
        outcome = check(load(args.main_report), load(args.branch_report))
    except (ReportError, KeyError, TypeError, ValueError) as e:
        print(f"family_net: {e}", file=sys.stderr)
        return 2
    print("\n".join(render(outcome)))
    return 0 if outcome.ok else 1


if __name__ == "__main__":
    sys.exit(main())
