"""The progress report generator (SPEC-0022) on a small synthetic report: the parity block, the
all-functions page and the charts (T-0081), input validation, the run record (T-0070), the
top-functions and evidence blocks (T-0073), and the usage-tiers block (T-5094).
test_progress_page.py covers the page and its judgment blocks, test_progress_history.py the run
history and stale runs (T-0071)."""

from __future__ import annotations

import dataclasses
import json
from pathlib import Path
from typing import Any

import pytest

from rustsheet_tools import progress_report
from rustsheet_tools.progress_report import evidence, history, page, parity, usage
from rustsheet_tools.progress_report.history import Git
from rustsheet_tools.progress_report.markdown import md_cell, md_text, pct, table
from rustsheet_tools.progress_report.model import (
    TIERS,
    Parity,
    ProgressReportError,
    compute,
    known_failures,
    load_usage_gates,
    run_of,
    tasks_of,
)
from rustsheet_tools.progress_report.svg import TIER_PALETTE, Bar, bar_chart, nice_step

SCOPE: dict[str, Any] = {
    "functions": [
        {"name": "ABS", "category": "math", "milestone": "M2", "scope": "in"},
        {"name": "SUM", "category": "math", "milestone": "M2", "scope": "in"},
        {"name": "LN", "category": "math", "milestone": "M5", "scope": "in"},
        {"name": "BETA.DIST", "category": "statistical", "milestone": "M9", "scope": "in"},
        {"name": "UPPER", "category": "text", "milestone": "M5", "scope": "in"},
        {"name": "INFO", "category": "information", "milestone": "M5"},
        {"name": "CALL", "category": "external", "milestone": "M14", "scope": "out"},
    ]
}

ZERO = {"pass": 0, "ulp1": 0, "ulp4": 0, "digits15": 0, "visible_numeric": 0, "visible_other": 0}
NO_BASELINES = Path("/nonexistent/eval/baseline")


def fn(name: str, tier: str, attribution: str = "sole", **attributed: int) -> dict[str, Any]:
    """A report `functions[]` entry (T-0068): its tier and attributed cells; its Excel cells are
    the attributed ones plus 10 bit-exact cells it shares with other functions."""
    cells = {**ZERO, **attributed}
    return {
        "name": name,
        "tier": tier,
        "attribution": attribution,
        "attributed": cells,
        "excel": {**cells, "pass": cells["pass"] + 10},
        "real_world_calls": 0,
    }


def rate(name: str, cells: int, visible: int, meets: bool, covered: bool = False) -> dict[str, Any]:
    """A `v15_checkpoint.functions[]` entry (T-0080): its Excel cells (`cells`) are its rate cells."""
    return {
        "name": name,
        "cells": cells,
        "covered": covered,
        "rate_cells": cells,
        "rate_visible": visible,
        "meets_rate": meets,
    }


def exit_row(milestone: str, exited: bool) -> dict[str, Any]:
    """A `milestone_exit` row (decision 0085): every measure met when `exited`."""
    return {
        "milestone": milestone,
        "functions": 3,
        "covered": 3,
        "coverage_met": True,
        "gen_scored": 20000,
        "gen_visibly_off": 4,
        "gen_allowance": 20,
        "gen_met": True,
        "calc_visibly_off": 0,
        "calc_met": True,
        "probe_visibly_off": 0 if exited else 2,
        "probes_met": exited,
        "exit": exited,
    }


# A docs/milestones.md: the phase table and the milestones the synthetic report and tasks name.
MILESTONES = """# Milestones

## Phase order (decision 0027)

| Phase | Milestones, in order | Status |
|---|---|---|
| 1. Parity (fifteen digits) | M0, M1, M2, M5, M6, M9 | active |
| 1b. Bit parity | the `defer: phase-1b` tasks of every milestone | after Phase 1 |
| 2. Architecture and performance | M3 | deferred |

## M0 — Scaffolding

**Scope:** the harness.
- [ ] not an exit criterion

**Exit criteria:**
- [x] CI green.
- [ ] Oracle set up,
  over two lines.
- [x] One loop merged.

**Eval suites:**
- [ ] not an exit criterion either

## M1 — Core semantics

**Exit criteria:**
- `smoke` at 100%.

## M2 — Top functions

**Exit criteria:** gen-m2 at 100%.

## M3 — Dependency graph (Phase 2)

**Exit criteria:**
- [ ] Metamorphic suite.

## M5 — Function families I

**Exit criteria:** catalog coverage 100%.

## M6 — Lookup & reference

**Exit criteria:** same as M5.

## M9 — Numerical parity

**Exit criteria:** ironcalc-calc at 99.95%.
"""


def report() -> dict[str, Any]:
    """ABS exact, SUM within 4 ulp, LN within 15 digits, UPPER at 99.9%, BETA.DIST below 99% placed
    by every Excel cell (fallback). INFO is called by no cell (unscored). CALL is out of scope."""
    tiers = {"exact": 1, "ulp4": 1, "digits15": 1, "rate999": 1, "rate99": 0, "below99": 1, "unscored": 1}
    fallback = {"below99": 1, "unscored": 1}
    return {
        "functions": [
            fn("ABS", "exact", **{"pass": 30}),
            fn("SUM", "ulp4", **{"pass": 20, "ulp1": 2, "ulp4": 1}),
            fn("LN", "digits15", **{"pass": 25, "digits15": 3}),
            fn("UPPER", "rate999", **{"pass": 1999, "visible_other": 1}),
            fn("BETA.DIST", "below99", "all", **{"pass": 4, "visible_numeric": 1}),
            fn("CALL", "below99", "all", visible_other=1),
        ],
        "tiers": [{"tier": t, "functions": tiers[t], "fallback": fallback.get(t, 0)} for t in TIERS],
        "real_world_suites": ["calipers", "poi"],
        "argument_coverage": [
            {"name": "ABS", "single": 0.5, "pairwise": None},
            {"name": "PI", "single": None, "pairwise": None},
        ],
        "v15_checkpoint": {
            "in_scope": 6,
            "at_rate": 4,
            "phase1_met": False,
            "at_strict_rate": 3,
            "met": False,
            "covered": 5,
            "visibly_off": 3,
            "rate_num": 999,
            "rate_den": 1000,
            "phase1_line": "Phase 1 (decision 0114): not met (4/6 functions covered)",
            "verdict_line": "Fifteen-digit parity: not met (coverage 5/6, visibly-off cells 3)",
            "functions": [
                rate("ABS", 30, 0, True),
                rate("SUM", 23, 0, True),
                rate("LN", 28, 0, True),
                rate("UPPER", 2000, 1, True, covered=True),
                rate("BETA.DIST", 15, 1, False),
                rate("INFO", 0, 0, False),
            ],
        },
        "cases": [],
        "milestone_exit": [
            exit_row("M5", True),
            {**exit_row("M6", False), "gen_scored": None, "gen_met": None},
        ],
        "milestone_lists": [
            {"name": "m1-probes", "cases": 9, "cases_run": 9, "known_failures": 2, "failing_cases": ["a"]}
        ],
        "probe_milestones": [
            {"milestone": "M0-M2", "known_failures": 3, "visibly_off": 1},
            {"milestone": "M5", "known_failures": 0, "visibly_off": 0},
        ],
        "catalog_coverage": [
            {"milestone": "M2", "functions": 2, "covered": 2},
            {"milestone": "M6", "functions": 0, "covered": 0},
        ],
        "run": {"commit": "abc12345", "date": "2026-10-19", "skipped_suites": []},
        "suites": [
            {
                "name": "s",
                "cases": 4,
                "cells": 100,
                "cells_passed": 90,
                "deviations": 2,
                "statuses": {"pass": 2, "fail": 1, "timeout": 1},
                "verdicts": {"pass": 88, "deviation": 2, "numeric_ulp": 4, "numeric_close": 1},
                "closeness": {**ZERO, "pass": 90, "visible_numeric": 1, "visible_other": 2},
                "workbooks": {"scored": 3, "all_exact": 1, "all_within15": 2, "any_visible": 1},
            }
        ],
    }


# Catalog priors (eval/catalog/functions.yaml `prior`): ABS, LN and UPPER tie; INFO has none.
PRIORS: dict[str, float | None] = {
    "SUM": 72.0,
    "ABS": 3.0,
    "UPPER": 3.0,
    "LN": 3.0,
    "BETA.DIST": 0.5,
    "INFO": None,
    "CALL": None,
}


def data(
    r: dict[str, Any] | None = None,
    baselines: Path = NO_BASELINES,
    priors: dict[str, float | None] | None = PRIORS,
) -> Parity:
    return compute(r or report(), SCOPE, baselines, priors)


def test_functions_come_from_the_catalog_with_the_reports_tiers() -> None:
    p = data()
    f = {x.name: x for x in p.functions}
    assert sorted(f) == ["ABS", "BETA.DIST", "INFO", "LN", "SUM", "UPPER"]
    assert (f["BETA.DIST"].tier, f["BETA.DIST"].attribution) == ("below99", "all")
    assert (f["INFO"].tier, f["INFO"].attribution, f["INFO"].excel_cells) == ("unscored", "all", 0)
    assert (f["UPPER"].attributed_cells, f["UPPER"].excel_cells, f["UPPER"].rate_within15) == (
        2000,
        2010,
        1999,
    )


def test_tier_table() -> None:
    text = parity.tier_table(data())
    assert "| Exact | every cell bit-exact | 1 | 16.67% | 0 |" in text
    assert "| &lt; 99% | &lt; 99% of cells not visibly off | 1 | 16.67% | 1 |" in text
    assert "| Unscored | no scored Excel cell | 1 | 16.67% | 1 |" in text
    assert "| **All** |  | **6** |  | 2 |" in text
    summary = parity.tier_summary(data())
    assert summary.startswith("**3 of 6** in-scope functions (50.00%) never differ")
    assert "**1** have fewer than 99%" in summary


def test_category_table() -> None:
    text = parity.category_table(data())
    lines = text.splitlines()
    assert lines[0] == "| Category | Functions | Exact | \N{LESS-THAN OR EQUAL TO} 4 ulp | 15 digits | " + (
        "\N{GREATER-THAN OR EQUAL TO} 99.9% | \N{GREATER-THAN OR EQUAL TO} 99% | &lt; 99% | Unscored |"
    )
    # The largest category first, then by name.
    assert lines[2] == "| math | 3 | 1 | 1 | 1 | 0 | 0 | 0 | 0 |"
    assert [line.split(" | ")[0] for line in lines[3:]] == ["| information", "| statistical", "| text"]


def test_attributed_cells_by_closeness() -> None:
    p = data()
    cells = p.attributed_cells()
    assert sum(cells.values()) == 30 + 23 + 28 + 2000 + 5
    assert (cells["visible_numeric"], cells["visible_other"]) == (1, 1)
    assert "| Visibly off: number | 1 | 0.05% |" in parity.closeness_table(p)
    assert parity.closeness_summary(p).startswith("**2,086** attributed Excel cells")
    # Decision 0310: a report from before `r3_math` shows none, on its own line and in the sentence.
    assert cells["r3_math"] == 0
    assert "| Math over Excel by design (decision 0188 R3) | 0 | 0.00% |" in parity.closeness_table(p)
    assert "0 math over Excel by design (decision 0188 R3)" in parity.closeness_summary(p)


def test_phase1_lines_come_from_v15_checkpoint() -> None:
    text = parity.phase1_lines(data())
    assert text.startswith("**Phase 1 (decision 0114): not met.** **4 of 6** in-scope functions")
    assert "_Information, gates nothing:_ 3 of 6 functions" in text


def test_suites_table_with_known_failures(tmp_path: Path) -> None:
    (tmp_path / "s.txt").write_text(
        "# header\nC\tcase\t5\nF\tcase\tA1\nF\tcase\tA2\nTC\tcase\t1\nTF\tcase\tB1\n"
    )
    assert known_failures(tmp_path / "s.txt") == (2, 1)
    assert known_failures(tmp_path / "none.txt") == (None, 0)
    row = parity.suites_table(data(baselines=tmp_path)).splitlines()[2]
    # Bit-exact 90/98, within 4 ulp 94/98, 3 visibly off, workbooks 3/1/2/1, 1 timeout.
    assert row == "| s | 4 | 100 | 91.84% | 95.92% | 3 | 3 | 1 | 2 | 1 | 1 | 2; text 1 |"
    assert parity.suites_table(data()).splitlines()[2].endswith("| no baseline |")


@pytest.mark.parametrize(
    ("mutate", "error"),
    [
        (lambda r: r["functions"][0].pop("tier"), "function 'ABS' has no `tier`; the report predates T-0068"),
        (
            lambda r: r["functions"][1].pop("attributed"),
            "'SUM' has no `attributed`; the report predates T-0068",
        ),
        (lambda r: r["functions"][2]["excel"].pop("digits15"), "excel has no `digits15`; .*T-0068"),
        (lambda r: r.pop("tiers"), "report.json has no `tiers`; the report predates T-0068"),
        (
            lambda r: r["suites"][0].pop("workbooks"),
            "suite 's' has no `workbooks`; the report predates T-0068",
        ),
        (lambda r: r.pop("v15_checkpoint"), "has no `v15_checkpoint`; the report predates T-0080"),
        (lambda r: r["v15_checkpoint"].pop("at_rate"), "has no `at_rate`; the report predates T-0080"),
        (lambda r: r["v15_checkpoint"]["functions"].pop(), "no entry for in-scope function INFO"),
        (lambda r: r["functions"][0].update(tier="great"), "tier is 'great', not one of"),
        (lambda r: r["tiers"][0].update(functions=2), "report.json `tiers` counts"),
        (lambda r: r["tiers"].pop(), "not every tier once in order"),
        (lambda r: r["suites"][0].update(cells="many"), "cells is not a count"),
        (lambda r: r["suites"][0].update(name="../x"), "not a suite name"),
        (lambda r: r.update(suites={}), "suites is not a list"),
    ],
)
def test_a_report_without_the_fields_this_reads_fails_loudly(mutate: Any, error: str) -> None:
    r = report()
    mutate(r)
    with pytest.raises(ProgressReportError, match=error):
        data(r)


def test_pct_never_rounds_to_the_ends() -> None:
    assert pct(9999, 10000) == "99.99%"
    assert pct(99999, 100000) == "99.99%"
    assert pct(1, 100000) == "0.01%"
    assert pct(5, 5) == "100.00%"
    assert pct(0, 0) == "\N{EN DASH}"


def catalog_yaml(priors: dict[str, Any]) -> str:
    """A functions.yaml holding one entry per name, with its prior when it is not None."""
    rows = [
        f"  - {{ name: {json.dumps(n)}" + ("" if p is None else f", prior: {p}") + " }"
        for n, p in priors.items()
    ]
    return "functions:\n" + "\n".join(rows) + "\n"


def stage(todo: int, blocked: int, done: int) -> dict[str, int]:
    """A tasks.json stage map (`scripts/tasks.py stats --json`)."""
    return {"todo": todo, "in-progress": 1, "blocked": blocked, "done": done, "dropped": 1, "ready": 2}


def tasks() -> dict[str, Any]:
    """A tasks.json: two milestones, the three campaign stages and the deferred tasks."""
    milestone = {"todo": 3, "in-progress": 1, "blocked": 2, "done": 10, "dropped": 1}
    return {
        "by_milestone": {"M0": milestone, "M2": {**milestone, "done": 5}},
        "by_campaign": {"semantics": stage(4, 1, 7), "accuracy": stage(2, 0, 1), "visible": stage(3, 2, 9)},
        "deferred": stage(5, 3, 0),
    }


class FakeGit:
    """`Git` with a fixed HEAD and commit counts by (since, until)."""

    def __init__(
        self, head: str | None = "abc12345", between: dict[tuple[str, str], int] | None = None
    ) -> None:
        self._head = head
        self._between = between or {}

    def head(self) -> str | None:
        return self._head

    def commits_between(self, since: str, until: str) -> int | None:
        return self._between.get((since, until))


GIT: Git = FakeGit()


def write_inputs(
    tmp_path: Path, r: dict[str, Any], t: dict[str, Any] | None = None, *, today: str = "2026-10-19"
) -> list[str]:
    rep, scope, tasks_json = tmp_path / "report.json", tmp_path / "scope.json", tmp_path / "tasks.json"
    catalog = tmp_path / "functions.yaml"
    milestones = tmp_path / "milestones.md"
    rep.write_text(json.dumps(r))
    scope.write_text(json.dumps(SCOPE))
    tasks_json.write_text(json.dumps(t or tasks()))
    catalog.write_text(catalog_yaml(PRIORS))
    milestones.write_text(MILESTONES)
    return [
        *("--report", str(rep), "--scope", str(scope), "--catalog", str(catalog)),
        *("--baselines", str(NO_BASELINES)),
        *("--tasks", str(tasks_json), "--milestones", str(milestones), "--today", today),
    ]


def test_main_writes_the_page_the_functions_page_and_the_charts(tmp_path: Path) -> None:
    out = tmp_path / "out"
    progress_report.main([*write_inputs(tmp_path, report()), "--out-dir", str(out)], GIT)
    page = (out / "progress.md").read_text()
    assert "## Parity" in page
    assert "(progress/functions.md)" in page
    for chart in parity.CHARTS:
        assert f"(progress/{chart.name})" in page
        svg = (out / "progress" / chart.name).read_text()
        assert "@media (prefers-color-scheme: dark)" in svg
        assert all(" fill=" in shape for shape in svg.split("<rect")[1:])
    functions = (out / "progress" / "functions.md").read_text()
    assert "2026-10-19" in functions
    assert "`abc12345`" in functions
    rows = [line for line in functions.splitlines() if line.startswith("| ")]
    names = [r.split(" | ")[0] for r in rows[2:]]
    assert names == ["| ABS", "| BETA.DIST", "| INFO", "| LN", "| SUM", "| UPPER"]
    assert all(r.count("|") == 10 for r in rows)
    assert "| BETA.DIST | statistical | M9 | &lt; 99% | every call | 5 | 15 | 93.33% | no |" in functions
    assert "| INFO | information | M5 | Unscored | every call | 0 | 0 | \N{EN DASH} | no |" in functions
    assert "## The 40 most-used functions" in page
    assert "## Evidence depth" in page
    assert f"(progress/{evidence.CHART})" in page
    assert "@media (prefers-color-scheme: dark)" in (out / "progress" / evidence.CHART).read_text()
    # The retired parity page is never written (T-0081: docs/parity.md is a static pointer).
    assert not (out / "parity.md").exists()
    assert not (out / "parity").exists()


def test_nice_step_picks_1_2_5_steps() -> None:
    assert [nice_step(t) for t in (0, 3, 5, 6, 100, 149, 238, 258_865)] == [1, 1, 1, 2, 20, 50, 50, 100_000]


def test_stacked_inner_labels_need_room() -> None:
    # 60 of 100 is about 280px wide: labelled. 1 of 100 is under MIN_INNER_LABEL_W: not labelled.
    svg = bar_chart("t", "d", [Bar("cat", (("pass", 60), ("ulp4", 1), ("visible_numeric", 39)), "100")])
    assert 'class="num in-pass"' in svg
    assert ">60</text>" in svg
    assert 'class="num in-visible_numeric"' in svg
    assert 'class="num in-ulp4"' not in svg
    # A one-segment bar has no inner label, only its end label.
    single = bar_chart("t", "d", [Bar("Off", (("visible_numeric", 7),), "7 (1%)")])
    assert "in-visible_numeric" not in single.split("</style>")[1]
    assert ">7 (1%)</text>" in single


def test_inner_label_ink_follows_the_theme() -> None:
    svg = bar_chart("t", "d", [Bar("c", (("digits15", 50), ("pass", 50)), "100")])
    light, dark = svg.split("@media (prefers-color-scheme: dark)")
    assert ".in-digits15{fill:#ffffff}" in light
    assert ".in-digits15{fill:#000000}" in dark


def test_the_tier_charts_use_the_tier_palette() -> None:
    p = data()
    svgs = {c.name: c.build(p, c.title) for c in parity.CHARTS}
    stacked = svgs["tiers-by-category.svg"]
    for label in parity.TIER_LABELS.values():
        assert f">{label.replace('<', '&lt;')}</text>" in stacked
    assert ">math</text>" in stacked
    assert f'class="b-below99" fill="{TIER_PALETTE.fills["below99"][0]}"' in stacked
    assert ">1 (16.67%)</text>" in svgs["tiers.svg"]
    assert ">Within 2\N{EN DASH}4 ulp</text>" in svgs["cells-by-closeness.svg"]
    assert set(TIER_PALETTE.fills) == set(TIERS) == set(TIER_PALETTE.ink)


def test_hostile_labels_are_escaped() -> None:
    hostile = '<script>"&|`\nx'
    svg = bar_chart(hostile, hostile, [Bar(hostile, (("pass", 1),), hostile)])
    assert "<script>" not in svg
    assert "&lt;script&gt;&quot;&amp;|" in svg
    md = table(["h"], [[hostile]])
    assert "<script>" not in md
    assert md.splitlines()[2] == '| &lt;script&gt;"&\\|&#96; x |'
    assert md_cell("a|b") == "a\\|b"
    # md_text writes `&` as an entity before md_cell adds its own, and escapes a backslash.
    assert md_text("&lt; a\\*b <c>") == "&amp;lt; a\\\\\\*b &lt;c&gt;"


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("commit", "abc; rm -rf /"),
        ("commit", "ABC12345"),
        ("date", "19 Oct"),
        ("date", None),
        ("skipped_suites", [{"name": "poi"}]),
        ("skipped_suites", None),
    ],
)
def test_the_run_record_is_validated(tmp_path: Path, field: str, value: object) -> None:
    # The run's commit, date and skipped suites are printed into the pages.
    r = report()
    r["run"][field] = value
    with pytest.raises(ProgressReportError, match=f"run.{field}"):
        run_of(r)
    with pytest.raises(SystemExit, match=field):
        progress_report.main([*write_inputs(tmp_path, r), "--out-dir", str(tmp_path / "out")], GIT)
    assert not (tmp_path / "out").exists()


def with_cells(cells: dict[str, int]) -> Parity:
    """`data()` with the Excel cells (`v15_checkpoint.functions[].cells`) of each named function."""
    r = report()
    for f in r["v15_checkpoint"]["functions"]:
        f["cells"] = cells[f["name"]]
    return data(r)


EDGES = {"INFO": 0, "ABS": 19, "SUM": 20, "LN": 399, "UPPER": 400, "BETA.DIST": 1000}


def test_evidence_bands_at_their_edges() -> None:
    p = with_cells(EDGES)
    assert evidence.band_counts(p) == [
        ("< 20", 2),
        ("20\N{EN DASH}199", 1),
        ("200\N{EN DASH}399", 1),
        ("400\N{EN DASH}999", 1),
        ("\N{GREATER-THAN OR EQUAL TO} 1000", 1),
    ]
    assert evidence.band_counts(with_cells({**EDGES, "ABS": 20, "LN": 400}))[:4] == [
        ("< 20", 1),
        ("20\N{EN DASH}199", 2),
        ("200\N{EN DASH}399", 0),
        ("400\N{EN DASH}999", 2),
    ]
    text = evidence.evidence_table(p)
    assert "| &lt; 20 | 2 | 33.33% |" in text
    assert "| **All** | **6** |  |" in text


def test_evidence_median_of_an_even_count() -> None:
    # Six functions: 0, 19, 20, 399, 400, 1000; the median is the mean of 20 and 399.
    p = with_cells(EDGES)
    assert evidence.median_cells(p) == 209.5
    block = evidence.evidence_block(p)
    assert "has **209.5** Excel cells" in block
    assert "**1 of\n6** (16.67%) are covered" in block
    assert (
        "**4** have fewer than\n400 Excel cells, 2 of them fewer than 20: INFO (0), ABS (19), SUM (20), LN (399)."
        in block
    )
    assert "has **400** Excel cells" in evidence.evidence_block(with_cells({**EDGES, "LN": 400, "SUM": 400}))


def test_top_functions_by_prior_ties_by_name() -> None:
    p = data()
    assert [f.name for f in evidence.top_functions(p)] == ["SUM", "ABS", "LN", "UPPER", "BETA.DIST", "INFO"]
    assert [f.name for f in evidence.top_functions(p, 3)] == ["SUM", "ABS", "LN"]
    lines = evidence.top_table(evidence.top_functions(p)).splitlines()
    assert lines[0] == (
        "| # | Function | Category | Tier | Prior | Attributed cells | Within 15 digits | Excel cells"
        " | Real-world calls |"
    )
    assert lines[2] == "| 1 | SUM | math | \N{LESS-THAN OR EQUAL TO} 4 ulp | 72 | 23 | 100.00% | 23 | 0 |"
    assert lines[6] == "| 5 | BETA.DIST | statistical | &lt; 99% | 0.5 | 5 | 93.33% | 15 | 0 |"
    assert lines[7] == "| 6 | INFO | information | Unscored | none | 0 | \N{EN DASH} | 0 | 0 |"
    block = evidence.top_block(p)
    assert "**3 of 6** never differ" in block
    assert "**4** meet the Phase 1 rate" in block


def test_the_catalog_priors_are_checked(tmp_path: Path) -> None:
    path = tmp_path / "functions.yaml"
    path.write_text(catalog_yaml(PRIORS))
    assert evidence.load_priors(path) == PRIORS
    path.write_text(catalog_yaml({"SUM": '"often"'}))
    with pytest.raises(ProgressReportError, match="prior that is not a non-negative number"):
        evidence.load_priors(path)
    with pytest.raises(ProgressReportError, match=r"functions\.yaml"):
        evidence.load_priors(tmp_path / "none" / "functions.yaml")
    without_ln = {k: v for k, v in PRIORS.items() if k != "LN"}
    with pytest.raises(ProgressReportError, match="the catalog has no function LN"):
        data(priors=without_ln)
    assert all(f.prior is None for f in data(priors=None).functions)


def test_real_world_line_weights_attributed_shares_by_calls() -> None:
    p = data()
    names = {s.name for s in p.suites}
    assert "calipers" not in names
    assert parity.real_world_line(p) == "Real-world calls: not measured, calipers and poi did not run."
    assert p.real_world_suites == ("calipers", "poi")
    corpora = [dataclasses.replace(p.suites[0], name=n) for n in p.real_world_suites]
    calls = {"SUM": 30, "BETA.DIST": 10, "INFO": 50}
    weighted = dataclasses.replace(
        p,
        suites=[*p.suites, *corpora],
        functions=[dataclasses.replace(f, real_world_calls=calls.get(f.name, 0)) for f in p.functions],
    )
    sum_, beta = (next(f for f in p.functions if f.name == n) for n in ("SUM", "BETA.DIST"))
    # INFO has no attributed cells, so its calls carry no share and stay out of the weights.
    share = (
        30 * sum_.attributed_within15 / sum_.attributed_cells
        + 10 * beta.attributed_within15 / beta.attributed_cells
    ) / 40
    assert (
        f"**{100 * share:.2f}%** of the 40 calls to attributed in-scope functions in the calipers and poi"
        in (parity.real_world_line(weighted))
    )
    assert parity.real_world_line(weighted) in parity.block(weighted)


def test_gate_lines_n_a_without_gate_b_gate_a_or_phase1b() -> None:
    text = parity.gate_lines(data())
    assert "Gate B (decision 0188 R1(b)): n/a (the report predates Gate B)." in text
    assert "Gate A (decision 0188 R2): n/a (no `eval/runs/accuracy.json`" in text
    assert "left out of the every-suite rate: n/a." in text
    assert "(4 of 6) is information" in text


def test_gate_lines_with_gate_b_gate_a_and_phase1b(tmp_path: Path) -> None:
    r = report()
    r["v15_checkpoint"]["gate_b"] = {
        "measured": True,
        "missing": [],
        "suites": [],
        "in_scope": 6,
        "at_rate": 5,
        "met": False,
        "functions": [],
    }
    r["v15_checkpoint"]["phase1b_left_out"] = 7
    acc = tmp_path / "accuracy.json"
    acc.write_text(
        json.dumps(
            {
                "passes": True,
                "holds": False,
                "check": {
                    "points": [
                        {"standing": "inside", "excel": {"t": "n", "v": 1.0}},
                        {"standing": "known_failure", "excel": {"t": "n", "v": 2.0}},
                    ],
                    "stale_known_failures": [],
                    "bad_marks": [],
                    "bad_exemptions": [],
                    "missing_tables": ["FOO"],
                },
            }
        )
    )
    p = compute(r, SCOPE, NO_BASELINES, PRIORS, acc)
    text = parity.gate_lines(p)
    assert "Gate B (decision 0188 R1(b)): not met, 5 of 6 functions at rate." in text
    assert "does not hold, 1 of 2 points inside the envelope, 1 known failures, 0 failing" in text
    assert "1 functions without a table" in text
    assert "left out of the every-suite rate: 7 visibly-off cells." in text


def test_gate_b_not_measured_names_the_missing_suites() -> None:
    r = report()
    r["v15_checkpoint"]["gate_b"] = {
        "measured": False,
        "missing": ["poi"],
        "in_scope": 6,
        "at_rate": 0,
        "met": False,
    }
    assert "n/a, 0 of 6 functions at rate; not measured, the run did not score poi" in parity.gate_lines(
        data(r)
    )


def ufn(
    name: str, use: str, cells: int, *, wrong: int = 0, outside: int = 0, meets: bool = True
) -> dict[str, Any]:
    """A report.json `usage_gates.functions[]` entry (SPEC-0037 design 3)."""
    rates = {
        k: (cells - n) / cells if cells else 1.0
        for k, n in (("type_rate", wrong), ("envelope_rate", outside))
    }
    return {
        "name": name,
        "usage": use,
        "cells": cells,
        "wrong_type": wrong,
        "outside": outside,
        **rates,
        "meets": meets,
    }


def utier(use: str, functions: list[dict[str, Any]], failing: list[str]) -> dict[str, Any]:
    """A `usage_gates.tiers[]` entry (not met) over `functions`, with decision 0355's bars for `use`."""
    mine = [f for f in functions if f["usage"] == use]
    cells, wrong, outside = (sum(f[k] for f in mine) for k in ("cells", "wrong_type", "outside"))
    top = use == "foundational"
    return {
        "usage": use,
        "functions": len(mine),
        "cells": cells,
        "wrong_type": wrong,
        "outside": outside,
        "type_rate": (cells - wrong) / cells,
        "envelope_rate": (cells - outside) / cells,
        "pooled_bar": {"type": 1.0, "envelope": 0.9999} if top else {"type": 0.999, "envelope": 0.999},
        "function_bar": {"type": 1.0, "envelope": 0.0} if top else {"type": 0.99, "envelope": 0.99},
        "failing_functions": failing,
        "met": False,
    }


def usage_gates() -> dict[str, Any]:
    """The synthetic report's tier gates: UPPER's 3 wrong-type cells fail the foundational tier
    (LN's one outside cell counts only pooled); BETA.DIST fails the expert per-function bars."""
    functions = [
        ufn("ABS", "foundational", 30),
        ufn("SUM", "foundational", 23),
        ufn("LN", "foundational", 28, outside=1),
        ufn("UPPER", "foundational", 2000, wrong=3, meets=False),
        ufn("BETA.DIST", "expert", 15, wrong=1, outside=1, meets=False),
        ufn("INFO", "expert", 0),
    ]
    tiers = [utier("foundational", functions, ["UPPER"]), utier("expert", functions, ["BETA.DIST"])]
    return {"suites": ["probes", "probes-numeric", "s"], "tiers": tiers, "functions": functions, "met": False}


def with_usage() -> dict[str, Any]:
    return {**report(), "usage_gates": usage_gates()}


# A `scripts/phase1_exit.py --json` output with the tier lines (SPEC-0037 design 5): each line
# keyed by `criterion` as today, and the old gates kept as `info`.
EXIT = {
    "overall": "not met",
    "failing": ["foundational", "expert"],
    "lines": [
        {"criterion": "run", "verdict": "met", "detail": "at abc12345: every suite ran, clean tree"},
        {
            "criterion": "foundational",
            "verdict": "not met",
            "detail": "3 wrong type, 1 outside; failing: UPPER",
        },
        {"criterion": "expert", "verdict": "not met", "detail": "failing: BETA.DIST"},
        {"criterion": "gate A", "verdict": "info", "detail": "2 points"},
        {"criterion": "function rate", "verdict": "info", "detail": "4/6"},
    ],
}
# The last complete history row, with its `usage` object.
LAST = {
    "date": "2026-10-18",
    "usage": {
        "foundational": {"cells": 2000, "wrong_type": 5, "outside": 1, "failing": 2, "met": False},
        "expert": {"cells": 15, "wrong_type": 1, "outside": 2, "failing": 1, "met": True},
    },
}
GE, MINUS = "\N{GREATER-THAN OR EQUAL TO}", "\N{MINUS SIGN}"
USAGE_BLOCK = f"""## Usage tiers

**Tier gates (decision 0355): foundational 3 wrong type, 1 outside, 1 failing function: not met; \
expert 1 wrong type, 1 outside, 1 failing function: not met.** Both: **not met**. They replace
Gate A and Gate B (decision 0188 R1 and R2), whose numbers below are information.

| Tier | Functions | Cells | Wrong type | Outside | Type rate | Bar | Envelope rate | Bar | Per-function bars | Met |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Foundational | 4 | 2,081 | 3 | 1 | 99.8558% | 100% | 99.9519% | {GE} 99.99% | type 100% | **not met** |
| Expert | 2 | 15 | 1 | 1 | 93.3333% | {GE} 99.9% | 93.3333% | {GE} 99.9% | type {GE} 99%, envelope {GE} 99% | **not met** |

Counted on the attributed cells (decision 0118) of probes, probes-numeric, s, without the decision 0134
exceptions, the deviations, the pivot suites and GETPIVOTDATA (report.json `usage_gates`). A type
miss is a wrong type, error or text; an envelope miss is a number outside 12 digits of Excel and
1e-9 of the math. Phase 1 exit tier lines (`make phase1-exit`, which adds the accuracy-table
points):

- **foundational: not met.** 3 wrong type, 1 outside; failing: UPPER
- **expert: not met.** failing: BETA.DIST

### Failing functions

Each tier's failing functions as the report lists them, worst first (the most cells wrong on
either measure), at most 10 per tier.

| Tier | Function | Cells | Wrong type | Outside | Type rate | Envelope rate |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Foundational | UPPER | 2,000 | 3 | 0 | 99.85% | 100.00% |
| Expert | BETA.DIST | 15 | 1 | 1 | 93.33% | 93.33% |

### Change since the last run

Against the last complete run, **2026-10-18**:

| Tier | Measure | 2026-10-18 | Now | Change |
| --- | --- | ---: | ---: | ---: |
| Foundational | Wrong type, error or text | 5 | 3 | {MINUS}2 |
| Foundational | Outside the envelope | 1 | 1 | 0 |
| Foundational | Failing functions | 2 | 1 | {MINUS}1 |
| Foundational | Met | not met | not met | unchanged |
| Expert | Wrong type, error or text | 1 | 1 | 0 |
| Expert | Outside the envelope | 2 | 1 | {MINUS}1 |
| Expert | Failing functions | 1 | 1 | 0 |
| Expert | Met | met | not met | no longer met |"""


def test_the_usage_tiers_block_is_pinned_on_a_fixture_report() -> None:
    assert usage.block(data(with_usage()).usage, EXIT, LAST) == USAGE_BLOCK


def test_a_report_without_usage_gates_shows_not_measured() -> None:
    assert data().usage is None
    assert usage.block(None, None, LAST) == (
        "## Usage tiers\n\n**Tier gates (decision 0355): not measured \\(report.json has no "
        "&#96;usage\\_gates&#96;: the report predates T-5092\\).** Until the report measures them, "
        "Gate A and\nGate B (decision 0188) still gate the Phase 1 exit.\n\n"
        "Phase 1 exit tier lines (`make phase1-exit`):\n\n"
        "- **foundational: not measured** (no phase1-exit.json beside the report).\n"
        "- **expert: not measured** (no phase1-exit.json beside the report)."
    )
    assert usage.headline(None).startswith("not measured (report.json has no `usage_gates`")
    # A phase1-exit.json from before T-5092 has no tier lines: each reads not measured.
    old = {
        "overall": "not met",
        "failing": [],
        "lines": [{"criterion": "gate A", "verdict": "met", "detail": ""}],
    }
    assert "- **expert: not measured** (phase1-exit.json has no `expert` line: it predates T-5092)." in (
        usage.exit_lines(old)
    )


def test_the_change_needs_an_earlier_row_with_the_tiers() -> None:
    u = data(with_usage()).usage
    assert u is not None
    for last in (None, {"date": "2026-10-18"}):
        assert "No earlier complete run in the history measured the tiers" in usage.since(u, last)


def test_failing_functions_are_worst_first_and_at_most_ten_per_tier() -> None:
    gates = usage_gates()
    # F01 to F12 have 1 to 12 wrong-type cells; F11B ties F11 on misses with fewer wrong-type cells.
    extra = [ufn(f"F{n:02}", "expert", 100, wrong=n, meets=False) for n in range(1, 13)]
    extra.append(ufn("F11B", "expert", 100, wrong=5, outside=6, meets=False))
    gates["functions"] += extra
    gates["tiers"][1]["failing_functions"] = ["BETA.DIST", *(f["name"] for f in extra)]
    gates["tiers"][0]["failing_functions"] = []
    u = load_usage_gates({"usage_gates": gates})
    assert u is not None
    order = [f.name for f in usage.failing(u, u.tiers[1])]
    assert order[:4] == ["F12", "F11", "F11B", "F10"]
    # BETA.DIST ties F02 on misses (2) with fewer wrong-type cells.
    assert order[-3:] == ["F02", "BETA.DIST", "F01"]
    text = usage.failing_table(u)
    rows = [line for line in text.splitlines() if line.startswith("| Expert")]
    assert len(rows) == 11
    assert rows[-1] == "| Expert | and 4 more |  |  |  |  |  |"
    assert "| Foundational | none |  |  |  |  |  |" in text


def test_failing_function_names_are_escaped_in_the_table() -> None:
    # T-5098: a name from the report is report text; a `|`, `_` or `*` in it must not break the
    # table or start emphasis, and `table`'s own md_cell must not escape it twice.
    gates = usage_gates()
    odd = ufn("A|B_C*", "expert", 100, wrong=3, meets=False)
    gates["functions"].append(odd)
    gates["tiers"][1]["failing_functions"] = ["A|B_C*"]
    u = load_usage_gates({"usage_gates": gates})
    assert u is not None
    row = next(line for line in usage.failing_table(u).splitlines() if "A\\|B" in line)
    assert "| A\\|B\\_C\\* |" in row
    assert row.replace("\\|", "").count("|") == 8  # 7 cells, so 8 unescaped pipes


@pytest.mark.parametrize(
    ("mutate", "error"),
    [
        (lambda g: g["tiers"].pop(), "has no expert tier"),
        (lambda g: g["tiers"].append(g["tiers"][0]), "lists foundational twice"),
        (lambda g: g["tiers"][0].update(usage="common"), "usage is 'common', not one of"),
        (lambda g: g["tiers"][0]["failing_functions"].append("NOPE"), "names NOPE, which"),
        (lambda g: g["tiers"][1]["pooled_bar"].update(type=99.9), "is not a share from 0 to 1"),
        (lambda g: g["functions"][0].pop("wrong_type"), "has no `wrong_type`"),
        (lambda g: g.pop("met"), "usage_gates has no `met`"),
    ],
)
def test_a_malformed_usage_gates_fails_loudly(mutate: Any, error: str) -> None:
    r = with_usage()
    mutate(r["usage_gates"])
    with pytest.raises(ProgressReportError, match=error):
        data(r)


def test_the_phase1_checklist_leads_with_the_tier_gates() -> None:
    first, gate_b, gate_a = parity.gate_lines(data(with_usage())).splitlines()[:3]
    assert first.startswith("- **Tier gates (decision 0355): foundational 3 wrong type, 1 outside")
    assert "They replace Gate B and Gate A as the Phase 1 gates" in first
    assert gate_b.startswith(
        "- **Gate B (decision 0188 R1(b)): n/a (the report predates Gate B).** _Information"
    )
    assert gate_a.endswith("_Information, gates nothing (decision 0355 item 5)._")
    unmeasured = parity.gate_lines(data())
    assert unmeasured.startswith("- **Tier gates (decision 0355): not measured")
    assert "Gate B and Gate A below still gate the Phase 1 exit" in unmeasured
    assert "_Information" not in unmeasured


def test_the_history_row_keeps_the_tiers_for_the_next_change() -> None:
    run, counts = run_of(report()), tasks_of(tasks())
    row = history.complete_row(run, data(with_usage()), counts, None)
    assert row["usage"] == {
        "foundational": {"cells": 2081, "wrong_type": 3, "outside": 1, "failing": 1, "met": False},
        "expert": {"cells": 15, "wrong_type": 1, "outside": 1, "failing": 1, "met": False},
    }
    assert history.parse(history.dump([row])) == [row]
    assert "usage" not in history.complete_row(run, data(), counts, None)
    bad = {**row, "usage": {"foundational": row["usage"]["foundational"]}}
    with pytest.raises(ProgressReportError, match="usage has no `expert`"):
        history.parse(history.dump([bad]))


def test_main_puts_the_usage_tiers_block_after_the_bottom_line(tmp_path: Path, capsys: Any) -> None:
    out, exit_json = tmp_path / "out", tmp_path / "phase1-exit.json"
    exit_json.write_text(json.dumps(EXIT))
    args = [*write_inputs(tmp_path, with_usage()), "--phase1-exit", str(exit_json), "--out-dir", str(out)]
    progress_report.main(args, GIT)
    text = (out / "progress.md").read_text()
    names = list(page.parse(text))
    assert names[names.index("bottom-line") + 1] == "usage-tiers"
    assert "- **foundational: not met.** 3 wrong type, 1 outside; failing: UPPER" in text
    assert "No earlier complete run in the history measured the tiers" in text
    assert "- **Usage tiers.** report.json `usage_gates`" in text
    assert (
        "usage tiers: foundational 3 wrong type, 1 outside, 1 failing function: not met;"
        in capsys.readouterr().out
    )
    # The next day's page shows the change against this run's history row.
    nxt = with_usage()
    nxt["run"] = {**nxt["run"], "date": "2026-10-20"}
    progress_report.main([*write_inputs(tmp_path, nxt), "--out-dir", str(out)], GIT)
    again = (out / "progress.md").read_text()
    assert "Against the last complete run, **2026-10-19**:" in again
    assert "| Expert | Met | not met | not met | unchanged |" in again
