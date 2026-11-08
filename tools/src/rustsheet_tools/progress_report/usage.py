"""The progress page's `usage-tiers` block: decision 0355's tier gates (SPEC-0037 design 7, T-5094).

The block follows the bottom line. It shows one row per usage tier, foundational and expert, with
its type and envelope rates against its bars and whether it is met (report.json `usage_gates`,
T-5092); the Phase 1 exit's two tier lines, which add the accuracy-table points
(`scripts/phase1_exit.py --json`, its `lines[]` entries whose `criterion` is `foundational` or
`expert`); the failing functions of each tier, worst first, at most ten; and the change since the
last complete history row that measured the tiers (its `usage` object, written here).

Every verdict and failing-function list is the report's or the exit script's; this module only
adds up their counts. A report without `usage_gates` shows "not measured".
"""

from __future__ import annotations

from typing import Any

from rustsheet_tools.progress_report.markdown import md_inline, md_text, pct, signed, table
from rustsheet_tools.progress_report.model import (
    EN,
    GE,
    USAGES,
    ProgressReportError,
    TierBar,
    UsageFunction,
    UsageGates,
    UsageTier,
    count,
    field,
    flag,
    items,
    mapping,
    string,
)

HEADING = "## Usage tiers"
# The failing functions listed per tier (SPEC-0037 design 7).
TOP_FAILING = 10
# Decimals of a pooled rate: the foundational envelope bar is 99.99%, so two would hide a miss.
POOLED_DECIMALS = 4
LABELS = {"foundational": "Foundational", "expert": "Expert"}
# A history row's `usage` object holds these counts per tier, and its verdict as `met`.
ROW_COUNTS = ("cells", "wrong_type", "outside", "failing")
DELTAS = (
    ("wrong_type", "Wrong type, error or text"),
    ("outside", "Outside the envelope"),
    ("failing", "Failing functions"),
)
NOT_MEASURED = "not measured (report.json has no `usage_gates`: the report predates T-5092)"


def verdict(met: bool) -> str:
    """`met` or `not met`."""
    return "met" if met else "not met"


def headline(u: UsageGates | None) -> str:
    """The tier gate line, which the steward's daily status leads with: per tier, its wrong-type and
    outside cells, its failing functions and its verdict ("foundational 31 wrong type, 1 outside,
    3 failing functions: not met; expert ...")."""
    if u is None:
        return NOT_MEASURED
    return "; ".join(
        f"{t.usage} {t.wrong_type:,} wrong type, {t.outside:,} outside, "
        f"{len(t.failing):,} failing function{'' if len(t.failing) == 1 else 's'}: {verdict(t.met)}"
        for t in u.tiers
    )


def _bar(share: float) -> str:
    """A pooled bar: `100%`, or `≥ 99.9%`."""
    return "100%" if share >= 1 else f"{GE} {100 * share:g}%"


def _function_bar(bar: TierBar) -> str:
    """The per-function bars; a 0 bar is none (the foundational envelope is pooled only)."""
    parts = [
        f"{name} {_bar(share)}" for name, share in (("type", bar.type), ("envelope", bar.envelope)) if share
    ]
    return ", ".join(parts) if parts else "none"


def tier_table(u: UsageGates) -> str:
    """One row per tier: its counts, rates against its bars, and the report's verdict."""
    rows = [
        [
            LABELS[t.usage],
            f"{t.functions:,}",
            f"{t.cells:,}",
            f"{t.wrong_type:,}",
            f"{t.outside:,}",
            pct(t.cells - t.wrong_type, t.cells, POOLED_DECIMALS),
            _bar(t.pooled_bar.type),
            pct(t.cells - t.outside, t.cells, POOLED_DECIMALS),
            _bar(t.pooled_bar.envelope),
            _function_bar(t.function_bar),
            f"**{verdict(t.met)}**",
        ]
        for t in u.tiers
    ]
    header = ["Tier", "Functions", "Cells", "Wrong type", "Outside", "Type rate", "Bar", "Envelope rate"]
    return table([*header, "Bar", "Per-function bars", "Met"], rows)


def failing(u: UsageGates, t: UsageTier) -> list[UsageFunction]:
    """The tier's failing functions (the report's list), worst first: the most cells wrong on
    either measure, then the most wrong-type cells, then by name."""
    fs = [u.functions[name] for name in t.failing]
    return sorted(fs, key=lambda f: (-f.misses, -f.wrong_type, f.name))


def failing_table(u: UsageGates) -> str:
    """At most TOP_FAILING failing functions per tier, worst first, with how many are left out."""
    rows = []
    for t in u.tiers:
        fs = failing(u, t)
        for f in fs[:TOP_FAILING]:
            rows.append(
                [
                    LABELS[t.usage],
                    md_inline(f.name),
                    f"{f.cells:,}",
                    f"{f.wrong_type:,}",
                    f"{f.outside:,}",
                    pct(f.cells - f.wrong_type, f.cells),
                    pct(f.cells - f.outside, f.cells),
                ]
            )
        if len(fs) > TOP_FAILING:
            rows.append([LABELS[t.usage], f"and {len(fs) - TOP_FAILING:,} more", "", "", "", "", ""])
        if not fs:
            rows.append([LABELS[t.usage], "none", "", "", "", "", ""])
    return table(
        ["Tier", "Function", "Cells", "Wrong type", "Outside", "Type rate", "Envelope rate"], rows, 2
    )


def exit_lines(exit_verdict: dict[str, Any] | None) -> str:
    """The Phase 1 exit's tier lines: the `lines[]` entries of `scripts/phase1_exit.py --json`
    whose `criterion` is a tier, each `verdict` and `detail` as the script wrote them."""
    if exit_verdict is None:
        return "\n".join(f"- **{u}: not measured** (no phase1-exit.json beside the report)." for u in USAGES)
    where = "phase1-exit.json"
    by_criterion: dict[str, Any] = {}
    for i, line in enumerate(items(exit_verdict, "lines", where)):
        by_criterion.setdefault(string(line, "criterion", f"{where} lines[{i}]"), line)
    out = []
    for u in USAGES:
        line = by_criterion.get(u)
        if line is None:
            out.append(f"- **{u}: not measured** (phase1-exit.json has no `{u}` line: it predates T-5092).")
            continue
        lwhere = f"{where} line {u!r}"
        detail = field(line, "detail", lwhere)
        if not isinstance(detail, str):
            msg = f"{lwhere}.detail is not a string: {detail!r}"
            raise ProgressReportError(msg)
        tail = f" {md_text(detail)}" if detail else ""
        out.append(f"- **{u}: {md_text(string(line, 'verdict', lwhere))}.**{tail}")
    return "\n".join(out)


def row(u: UsageGates) -> dict[str, Any]:
    """The history row's `usage` object: per tier, its counts and verdict."""
    return {
        t.usage: {
            "cells": t.cells,
            "wrong_type": t.wrong_type,
            "outside": t.outside,
            "failing": len(t.failing),
            "met": t.met,
        }
        for t in u.tiers
    }


def check_row(obj: Any, where: str) -> None:
    """Check a history row's optional `usage` object; rows without one are valid."""
    for u in USAGES:
        per = mapping(obj, u, where)
        for key in ROW_COUNTS:
            count(per, key, f"{where}.{u}")
        flag(per, "met", f"{where}.{u}")


def _change(before: bool, now: bool) -> str:
    if before == now:
        return "unchanged"
    return "now met" if now else "no longer met"


def since(u: UsageGates, last: dict[str, Any] | None) -> str:
    """The change against `last`, the last complete history row, when it measured the tiers."""
    before = last.get("usage") if last else None
    if last is None or before is None:
        return "No earlier complete run in the history measured the tiers, so there is no change to show yet."
    now = row(u)
    rows = []
    for t in USAGES:
        rows += [
            [
                LABELS[t],
                label,
                f"{before[t][key]:,}",
                f"{now[t][key]:,}",
                signed(now[t][key] - before[t][key]),
            ]
            for key, label in DELTAS
        ]
        rows.append(
            [
                LABELS[t],
                "Met",
                verdict(before[t]["met"]),
                verdict(now[t]["met"]),
                _change(before[t]["met"], now[t]["met"]),
            ]
        )
    return f"Against the last complete run, **{last['date']}**:\n\n" + table(
        ["Tier", "Measure", last["date"], "Now", "Change"], rows, 2
    )


def block(u: UsageGates | None, exit_verdict: dict[str, Any] | None, last: dict[str, Any] | None) -> str:
    """The `usage-tiers` block; `last` is the last complete history row before this run."""
    lines = exit_lines(exit_verdict)
    if u is None:
        return f"""{HEADING}

**Tier gates (decision 0355): {md_text(NOT_MEASURED)}.** Until the report measures them, Gate A and
Gate B (decision 0188) still gate the Phase 1 exit.

Phase 1 exit tier lines (`make phase1-exit`):

{lines}"""
    suites = ", ".join(md_text(s) for s in u.suites) or "no suite"
    return f"""{HEADING}

**Tier gates (decision 0355): {md_text(headline(u))}.** Both: **{verdict(u.met)}**. They replace
Gate A and Gate B (decision 0188 R1 and R2), whose numbers below are information.

{tier_table(u)}

Counted on the attributed cells (decision 0118) of {suites}, without the decision 0134
exceptions, the deviations, the pivot suites and GETPIVOTDATA (report.json `usage_gates`). A type
miss is a wrong type, error or text; an envelope miss is a number outside 12 digits of Excel and
1e-9 of the math. Phase 1 exit tier lines (`make phase1-exit`, which adds the accuracy-table
points):

{lines}

### Failing functions

Each tier's failing functions as the report lists them, worst first (the most cells wrong on
either measure), at most {TOP_FAILING} per tier.

{failing_table(u)}

### Change since the last run

{since(u, last)}"""


METHOD = f"""- **Usage tiers.** report.json `usage_gates` (crates/rustsheet-eval, SPEC-0037 design 3):
  each tier's counts, bars, verdict and failing functions as the report writes them. A rate is
  (cells {EN} misses) / cells, shown to {POOLED_DECIMALS} decimals for a tier and never rounded
  to 100%. The exit tier lines are the `foundational` and `expert` lines of
  eval/runs/latest/phase1-exit.json. The change is against the last complete history row that
  has a `usage` object."""
