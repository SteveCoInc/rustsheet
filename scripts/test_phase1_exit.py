"""Unit tests for scripts/phase1_exit.py (stdlib only; run by `make hygiene`)."""

from __future__ import annotations

import io
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import phase1_exit  # noqa: E402 (the sys.path line above must run first)

GEN = "gen-m9/0123456789abcdef.case.json"
TSV = "# provenance\nname\tms_category\tmin_args\tsyntax\tpage\tnote\nSUM\tMath\t1\tSUM(x)\thttps://x\t\nN\tInfo\t1\tN(x)\thttps://x\t\nCALL\tAdd-in\t1\tCALL(x)\thttps://x\t\n"
YAML = (
    "functions:\n"
    "  - name: SUM\n    category: math\n    milestone: M2\n    scope: in\n"
    '  - name: "N"\n    category: information\n    milestone: M5\n'
    '  - { name: CALL, category: addin, milestone: M5, scope: out, reason: "add-in" }\n'
)


def gate_function(name: str, usage: str, cells: int, wrong: int = 0, outside: int = 0) -> dict[str, Any]:
    """One `usage_gates.functions` row (its rates and `meets` are not read here)."""
    return {"name": name, "usage": usage, "cells": cells, "wrong_type": wrong, "outside": outside}


def usage_gates(*functions: dict[str, Any]) -> dict[str, Any]:
    """report.json `usage_gates` (SPEC-0037 design 3) with decision 0355's bars; SUM foundational
    and IRR expert, all right, by default."""
    rows = list(functions) or [gate_function("SUM", "foundational", 100), gate_function("IRR", "expert", 1000)]
    bars = {
        "foundational": ({"type": 1.0, "envelope": 0.9999}, {"type": 1.0, "envelope": 0.0}),
        "expert": ({"type": 0.999, "envelope": 0.999}, {"type": 0.99, "envelope": 0.99}),
    }
    return {
        "suites": ["probes"],
        "tiers": [{"usage": u, "pooled_bar": b[0], "function_bar": b[1]} for u, b in bars.items()],
        "functions": rows,
        "met": True,
    }


def point(table: str, cls: str = "both", engine: Any = None) -> dict[str, Any]:
    """An accuracy point of `table` in envelope class `cls`."""
    standing = "inside" if cls in ("excel", "math", "both") else "known_failure"
    return {"table": table, "class": cls, "standing": standing, "engine": engine or {"t": "n", "v": 1.0}}


def report(**over: Any) -> dict[str, Any]:
    """A report every measured criterion of which is met."""
    base: dict[str, Any] = {
        "usage_gates": usage_gates(),
        "run": {"commit": "abc12345", "dirty": False, "skipped_suites": []},
        "v15_checkpoint": {
            "phase1_met": True,
            "phase1_line": "Phase 1 (decision 0114): met",
            "stale_exceptions": [],
            "unchecked_exceptions": [],
        },
        "milestone_exit": [{"milestone": m, "exit": True} for m in phase1_exit.PHASE1_MILESTONES],
        "out_of_scope": [{"name": "CALL", "reason": "add-in"}],
        "cases": [],
        "attributed_off": [],
        "unattributed_off": [],
        "probe_milestones": [],
    }
    base.update(over)
    return base


def accuracy(holds: bool = True, **check: Any) -> dict[str, Any]:
    """The accuracy check's JSON (`rustsheet-eval accuracy --json`), Gate A holding by default."""
    base: dict[str, Any] = {
        "points": [point("SUM"), point("IRR")],
        "stale_known_failures": [],
        "bad_marks": [],
        "bad_exemptions": [],
        "missing_tables": [],
        "seconds": 0.1,
    }
    base.update(check)
    return {"passes": True, "holds": holds, "check": base}


class Phase1Exit(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.tasks = root / "tasks"
        self.tasks.mkdir()
        self.catalog = root / "catalog"
        self.catalog.mkdir()
        (self.catalog / "excel-functions.tsv").write_text(TSV)
        (self.catalog / "functions.yaml").write_text(YAML)
        self.n = 0
        self.accuracy: dict[str, Any] | None = accuracy()

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def task(self, body: str, milestone: str = "M9", defer: bool = False) -> None:
        self.n += 1
        d = self.tasks / milestone
        d.mkdir(exist_ok=True)
        extra = "defer: phase-1b\n" if defer else ""
        (d / f"T-{9000 + self.n}-x.md").write_text(
            f"---\nid: T-{9000 + self.n}\nmilestone: {milestone}\nstatus: todo\n{extra}---\n\n{body}\n"
        )

    def run_main(self, rep: dict[str, Any]) -> tuple[int, str]:
        path = Path(self.tmp.name) / "report.json"
        path.write_text(json.dumps(rep))
        acc = Path(self.tmp.name) / "accuracy.json"
        if self.accuracy is not None:
            acc.write_text(json.dumps(self.accuracy))
            # Written after the report, as `make eval` does.
            mtime = path.stat().st_mtime + 1
            os.utime(acc, (mtime, mtime))
        out = io.StringIO()
        with redirect_stdout(out):
            code = phase1_exit.main(
                ["--report", str(path), "--tasks", str(self.tasks), "--catalog", str(self.catalog)]
                + ["--accuracy", str(acc)]
            )
        return code, out.getvalue()

    def failing(self, text: str) -> str:
        return text.strip().splitlines()[-1]

    def test_all_met_exits_0_and_lists_the_manual_items(self) -> None:
        code, text = self.run_main(report())
        self.assertEqual(code, 0, text)
        self.assertIn("catalog\tmet\t3 Excel functions, 2 in scope, 1 out", text)
        self.assertIn("foundational\tmet\t1 functions, 101 cells (1 accuracy points): type 100.0000%", text)
        self.assertIn("expert\tmet\t1 functions, 1001 cells (1 accuracy points)", text)
        self.assertIn(
            "gate A\tinfo\tmet (information, gates nothing): 2 points, 2 inside, 0 known failures, 0 failing", text
        )
        # The tier lines come right after the catalog, the old gates after them (decision 0355).
        criteria = [ln.split("\t")[0] for ln in text.splitlines()[:6]]
        self.assertEqual(criteria, ["run", "catalog", "foundational", "expert", "gate A", "function rate"])
        self.assertEqual(text.count("\tmanual\t"), len(phase1_exit.MANUAL_ITEMS))
        self.assertTrue(self.failing(text).startswith("Phase 1 exit: met"))

    def test_gate_a_not_holding_is_information_only(self) -> None:
        # Decision 0355 item 5: Gate A's verdict stays on the line, marked `info`, and gates
        # nothing; the tier gates read the same points.
        self.accuracy = accuracy(
            holds=False,
            points=[point("SUM"), point("IRR", "outside")],
            missing_tables=["ACCRINT", "AMORDEGRC"],
        )
        code, text = self.run_main(report())
        self.assertEqual(code, 0, text)
        self.assertIn(
            "gate A\tinfo\tnot met (information, gates nothing): 2 points, 1 inside, 1 known failures, 0 failing, "
            "0 list problems; missing tables 2",
            text,
        )
        # 1 outside of 1,001 expert cells is 99.9%: met.
        self.assertIn("expert\tmet\t1 functions, 1001 cells (1 accuracy points): type 100.0000% (0 wrong", text)
        self.assertIn("envelope 99.9001% (1 outside, bar ≥ 99.9%)", text)

    def test_a_foundational_wrong_type_fails_its_tier(self) -> None:
        # Decision 0355 item 3: foundational cells are 100% right on type, eval cells and accuracy
        # points alike; the failing functions are named with their counts.
        rep = report(usage_gates=usage_gates(gate_function("SUM", "foundational", 100, outside=1),
                                             gate_function("ABS", "foundational", 50, wrong=1),
                                             gate_function("IRR", "expert", 1000)))
        self.accuracy = accuracy(points=[point("SUM", "error_mismatch"), point("IRR")])
        code, text = self.run_main(rep)
        self.assertEqual(code, 1)
        line = next(ln for ln in text.splitlines() if ln.startswith("foundational\t"))
        self.assertIn("foundational\tnot met\t2 functions, 151 cells (1 accuracy points)", line)
        # SUM: 1 wrong (the point) and 1 outside; ABS 1 wrong: both below the per-function 100%.
        self.assertIn("failing (wrong type/outside): SUM 1/1, ABS 1/0;", line)
        self.assertIn("pooled misses from (wrong type/outside): SUM 1/1, ABS 1/0", line)
        self.assertEqual(self.failing(text), "Phase 1 exit: not met: foundational")

    def test_the_expert_bars_pooled_and_per_function(self) -> None:
        # 2 outside of 1,101 pooled is under 99.9%, though IRR meets its own 99%; RATE at 98% fails
        # its own bar. The failing functions are those below their own bar (gate_tiers.rs).
        functions = [
            gate_function("SUM", "foundational", 100),
            gate_function("IRR", "expert", 1000, outside=1),
            gate_function("RATE", "expert", 100, wrong=2),
        ]
        self.accuracy = accuracy(points=[point("IRR", "outside")])
        code, text = self.run_main(report(usage_gates=usage_gates(*functions)))
        self.assertEqual(code, 1)
        line = next(ln for ln in text.splitlines() if ln.startswith("expert\t"))
        self.assertIn("expert\tnot met\t2 functions, 1101 cells (1 accuracy points)", line)
        self.assertIn("failing (wrong type/outside): RATE 2/0;", line)
        self.assertIn("pooled misses from (wrong type/outside): IRR 0/2, RATE 2/0", line)
        self.assertTrue(phase1_exit.share_meets(100, 1, 0.99))
        self.assertFalse(phase1_exit.share_meets(100, 2, 0.99))
        self.assertTrue(phase1_exit.share_meets(1000, 1, 0.999))
        self.assertFalse(phase1_exit.share_meets(1000, 2, 0.999))
        self.assertTrue(phase1_exit.share_meets(10_000, 1, 0.9999))
        self.assertTrue(phase1_exit.share_meets(0, 0, 1.0))
        self.assertTrue(phase1_exit.share_meets(10, 9, 0.0), "0 is no bar")

    def test_an_accuracy_point_outside_with_an_error_is_a_wrong_type(self) -> None:
        self.assertEqual(phase1_exit.point_class(point("X", "outside", {"t": "e", "v": "#NUM!"})), "wrong_type")
        self.assertEqual(phase1_exit.point_class(point("X", "outside")), "outside")
        self.assertEqual(phase1_exit.point_class(point("X", "math")), "inside")
        self.assertIsNone(phase1_exit.point_class(point("X", "table")))

    def test_a_report_without_usage_gates_is_not_measured(self) -> None:
        rep = report()
        del rep["usage_gates"]
        code, text = self.run_main(rep)
        self.assertEqual(code, 1)
        self.assertIn("foundational\tn/a\tno usage_gates in the report", text)
        self.assertEqual(self.failing(text), "Phase 1 exit: not met: foundational, expert")

    def test_gate_a_absent_or_older_than_the_report_is_not_measured(self) -> None:
        self.accuracy = None
        code, text = self.run_main(report())
        self.assertEqual(code, 1)
        self.assertIn("gate A\tinfo\tn/a (information, gates nothing): no accuracy.json", text)
        # The tier gates need the accuracy points: unmeasured, never met.
        self.assertIn("foundational\tn/a\tno accuracy.json", text)
        self.assertEqual(self.failing(text), "Phase 1 exit: not met: foundational, expert")
        acc = Path(self.tmp.name) / "accuracy.json"
        rep = Path(self.tmp.name) / "report.json"
        acc.write_text(json.dumps(accuracy()))
        rep.write_text(json.dumps(report()))
        old = rep.stat().st_mtime - 10
        os.utime(acc, (old, old))
        line = phase1_exit.gate_a_line(acc, rep)
        self.assertEqual((line.verdict, line.detail), ("n/a", "accuracy.json is older than the report: run `make eval`"))

    def test_gate_a_unreadable_is_not_measured(self) -> None:
        self.accuracy = {"holds": True}
        code, text = self.run_main(report())
        self.assertEqual(code, 1)
        self.assertIn("gate A\tinfo\tn/a (information, gates nothing): accuracy.json unreadable", text)
        self.assertIn("expert\tn/a\taccuracy JSON unreadable", text)

    def test_a_milestone_row_not_met(self) -> None:
        rows = [{"milestone": m, "exit": m != "M7"} for m in phase1_exit.PHASE1_MILESTONES]
        code, text = self.run_main(report(milestone_exit=rows))
        self.assertEqual(code, 1)
        self.assertIn("milestone M7\tnot met", text)
        self.assertEqual(self.failing(text), "Phase 1 exit: not met: milestone M7")

    def test_a_missing_milestone_row_is_not_met(self) -> None:
        rows = [{"milestone": m, "exit": True} for m in phase1_exit.PHASE1_MILESTONES if m != "M10"]
        code, text = self.run_main(report(milestone_exit=rows))
        self.assertEqual(code, 1)
        self.assertIn("milestone M10\tnot met\tno milestone_exit row", text)

    def test_m16_is_not_a_phase1_milestone(self) -> None:
        # Decision 0354: M16 moves to Phase 2; a failing M16 row does not hold the Phase 1 exit.
        self.assertNotIn("M16", phase1_exit.PHASE1_MILESTONES)
        rows = [{"milestone": m, "exit": True} for m in phase1_exit.PHASE1_MILESTONES]
        code, text = self.run_main(report(milestone_exit=[*rows, {"milestone": "M16", "exit": False}]))
        self.assertNotIn("milestone M16", text)
        self.assertNotIn("milestone M16", self.failing(text))

    def test_the_m16_line_names_each_measure(self) -> None:
        m16 = {
            "milestone": "M16",
            "exit": False,
            "gen_suite": "gen-pivot",
            "gen_met": False,
            "more_suites": [{"name": "pivot-corpus-lo", "met": None}],
            "probes_met": True,
            "pivot": {
                "stages": [{"stage": "PV1", "rows": 10, "web_cannot_render": 1, "evidenced": 4}],
                "evidence_met": False,
                "owners_met": None,
                "layout_met": None,
                "corpus_render_met": None,
            },
        }
        self.assertEqual(
            phase1_exit.pivot_detail(m16),
            "gen-pivot not met; pivot-corpus-lo n/a; probes met; "
            "evidence not met (PV1 4/9 at the floor); owners n/a; layout n/a; corpus render n/a",
        )

    def test_phase1_met_false_is_information_only(self) -> None:
        code, text = self.run_main(
            report(v15_checkpoint={"phase1_met": False, "phase1_line": "not met (1/2)"})
        )
        self.assertEqual(code, 0, text)
        self.assertIn("function rate\tinfo\tnot met (information, gates nothing): not met (1/2)", text)

    def test_the_rate_reads_gate_b_met_not_met_or_not_measured(self) -> None:
        # Decision 0188 R1(b) (T-3428): the line still reads Gate B, met, not met or `n/a` with the
        # suites the run did not score; decision 0355 item 5 makes it information.
        v15 = dict(report()["v15_checkpoint"], gate_b={"measured": True, "met": True, "missing": []})
        code, text = self.run_main(report(v15_checkpoint=v15))
        self.assertEqual(code, 0, text)
        self.assertIn("function rate\tinfo\tmet (information", text)
        v15["gate_b"] = {"measured": True, "met": False, "missing": []}
        code, text = self.run_main(report(v15_checkpoint=v15))
        self.assertEqual(code, 0, text)
        self.assertIn("function rate\tinfo\tnot met (information", text)
        v15["gate_b"] = {"measured": False, "met": False, "missing": ["calipers", "poi"]}
        code, text = self.run_main(report(v15_checkpoint=v15))
        self.assertIn(
            "function rate\tinfo\tn/a (information, gates nothing): Gate B not measured: the run did not score "
            "calipers, poi",
            text,
        )
        v15["gate_b"] = ["not", "an", "object"]
        code, text = self.run_main(report(v15_checkpoint=v15))
        self.assertIn("function rate\tinfo\tn/a (information, gates nothing): Gate B unreadable", text)
        self.assertTrue(self.failing(text).startswith("Phase 1 exit: met"))

    def test_a_stale_exception_shows_on_the_rate_line(self) -> None:
        v15 = dict(report()["v15_checkpoint"], stale_exceptions=["gen-m9/x!Calc!A1"])
        code, text = self.run_main(report(v15_checkpoint=v15))
        self.assertEqual(code, 0, text)
        self.assertIn("function rate\tinfo\tnot met", text)
        self.assertIn("1 stale exception lines", text)

    def test_a_skipped_suite(self) -> None:
        run = {
            "commit": "abc12345",
            "dirty": False,
            "skipped_suites": [{"name": "poi", "reason": "not fetched"}],
        }
        code, text = self.run_main(report(run=run))
        self.assertEqual(code, 1)
        self.assertIn("run\tnot met\tat abc12345: skipped suites: poi", text)

    def test_a_dirty_run(self) -> None:
        code, text = self.run_main(report(run={"commit": "abc12345", "dirty": True, "skipped_suites": []}))
        self.assertEqual(code, 1)
        self.assertIn("dirty tree", text)

    def unowned_off_report(self) -> dict[str, Any]:
        off = {
            "function": "GAMMA",
            "case": GEN,
            "cell": "Calc!A1",
            "formula": "=GAMMA(2)",
            "expected": 1,
            "actual": 2,
        }
        return report(
            attributed_off=[off],
            cases=[{"id": GEN, "suite": "gen-m9", "cells": []}],
        )

    def test_one_unowned_cell(self) -> None:
        code, text = self.run_main(self.unowned_off_report())
        self.assertEqual(code, 1)
        self.assertIn("owners\tnot met\t1 visibly-off cells with no open owner (0 owned only", text)

    def test_an_owned_cell_is_met(self) -> None:
        self.task("gen-m9/0123456789abcdef Calc!A1 is off.")
        code, text = self.run_main(self.unowned_off_report())
        self.assertEqual(code, 0, text)

    def test_a_cell_owned_only_by_a_deferred_task_is_unowned(self) -> None:
        self.task("gen-m9/0123456789abcdef Calc!A1 is off.", defer=True)
        code, text = self.run_main(self.unowned_off_report())
        self.assertEqual(code, 1)
        self.assertIn("1 visibly-off cells with no open owner (1 owned only by a deferred task)", text)

    def test_a_catalog_gap(self) -> None:
        (self.catalog / "excel-functions.tsv").write_text(TSV + "ABS\tMath\t1\tABS(x)\thttps://x\t\n")
        code, text = self.run_main(report())
        self.assertEqual(code, 1)
        self.assertIn("catalog\tnot met\tnot in the catalog: ABS", text)

    def test_a_report_out_of_scope_list_that_differs(self) -> None:
        code, text = self.run_main(report(out_of_scope=[]))
        self.assertEqual(code, 1)
        self.assertIn("catalog\tnot met\treport out_of_scope differs: CALL", text)

    def test_the_real_catalog_parses_completely(self) -> None:
        # The line parser must keep up with functions.yaml's layout: a reformat that hides an entry
        # or its scope shows here, not as a silent `met` (tools/tests/test_catalog.py holds the
        # same set equality through PyYAML).
        line = phase1_exit.catalog_line(phase1_exit.CATALOG)
        self.assertEqual(line.verdict, "met", line.detail)
        scopes = phase1_exit.catalog_scopes(phase1_exit.CATALOG / "functions.yaml")
        self.assertIn("out", scopes.values())

    def test_json_output(self) -> None:
        path = Path(self.tmp.name) / "report.json"
        path.write_text(json.dumps(report()))
        out = io.StringIO()
        acc = Path(self.tmp.name) / "accuracy.json"
        acc.write_text(json.dumps(accuracy()))
        mtime = path.stat().st_mtime + 1
        os.utime(acc, (mtime, mtime))
        args = ["--report", str(path), "--tasks", str(self.tasks), "--catalog", str(self.catalog), "--json"]
        args += ["--accuracy", str(acc)]
        with redirect_stdout(out):
            self.assertEqual(phase1_exit.main(args), 0)
        doc = json.loads(out.getvalue())
        self.assertEqual(doc["overall"], "met")
        self.assertEqual(doc["failing"], [])
        self.assertEqual(doc["lines"][0]["criterion"], "run")
        # The dashboard reads the tier lines and the `info` lines as ordinary lines (T-5094).
        by = {ln["criterion"]: ln for ln in doc["lines"]}
        self.assertEqual((by["foundational"]["verdict"], by["expert"]["verdict"]), ("met", "met"))
        self.assertEqual((by["gate A"]["verdict"], by["function rate"]["verdict"]), ("info", "info"))
        self.assertEqual(set(doc), {"overall", "failing", "lines"})


if __name__ == "__main__":
    unittest.main()
