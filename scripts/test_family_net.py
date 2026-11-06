"""Tests for family_net.py, decision 0267 item 6's net-improvement check (T-4159)."""

from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import family_net  # noqa: E402 (the sys.path line above must run first)

FAMILY = ["BETA.DIST", "BINOM.DIST", "F.DIST"]


def cell(
    at: str, bucket: str, *, fam: bool = True, formula: str = "=BETA.DIST(A1,2,3,TRUE)"
) -> dict[str, Any]:
    """A listed cell as `closeness::serialize_cells` writes it."""
    out: dict[str, Any] = {
        "sheet": "S",
        "cell": at,
        "formula": formula,
        "verdict": "numeric_close" if bucket != "visible_other" else "type_mismatch",
        "closeness": bucket,
    }
    if fam:
        out["thirteen_digit_family"] = True
    return out


def report(
    cells: list[dict[str, Any]],
    within15: int,
    visible: int,
    *,
    checked: int = 10,
    text: list[dict[str, Any]] | None = None,
    status: str = "fail",
) -> dict[str, Any]:
    """A one-case report: `checked` cells, the unlisted ones bit-exact; BETA.DIST holds the family
    counts, and a non-family function's counts never matter."""
    case: dict[str, Any] = {
        "id": "c",
        "suite": "probes-numeric",
        "status": status,
        "checked": checked,
        "passed": checked - len(cells),
        "deviations": 0,
        "cells": cells,
        "text": text or [],
    }
    return {
        "suites": [{"name": "probes-numeric"}],
        "thirteen_digit_family": FAMILY,
        "functions": [
            {
                "name": "BETA.DIST",
                "closeness": {"pass": within15, "visible_numeric": visible, "residue13": 2},
            },
            {"name": "T.DIST", "closeness": {"pass": 1000, "visible_numeric": 1000}},
        ],
        "cases": [case],
    }


class FamilyNet(unittest.TestCase):
    def test_a_clean_improvement(self) -> None:
        # A1 regresses from bit-exact to residue13, A2 from digits15 to residue13; A3 becomes
        # bit-exact (it is listed on main only), and the family gains.
        main = report([cell("A2", "digits15"), cell("A3", "visible_numeric")], 100, 5)
        branch = report([cell("A1", "residue13"), cell("A2", "residue13")], 101, 4)
        o = family_net.check(main, branch)
        self.assertTrue(o.ok)
        self.assertEqual([r.cell for r in o.regressions], ["c!S!A1", "c!S!A2"])
        self.assertTrue(all(r.allowed for r in o.regressions))
        self.assertEqual((o.within15, o.visible_numeric, o.compared_cases), ((100, 101), (5, 4), 1))
        lines = family_net.render(o)
        self.assertIn("  family within 15 digits (pass+ulp1+ulp4+digits15): 100 -> 101 (+1)", lines)
        self.assertIn("  family visible_numeric: 5 -> 4 (-1)", lines)
        self.assertIn("    c!S!A1\tpass -> residue13\t=BETA.DIST(A1,2,3,TRUE)", lines)

    def test_a_non_family_regression(self) -> None:
        main = report([], 100, 5)
        branch = report([cell("A1", "ulp1", fam=False, formula="=T.DIST(1,2,TRUE)")], 101, 5)
        o = family_net.check(main, branch)
        self.assertFalse(o.ok)
        self.assertEqual([r.cell for r in o.rejected], ["c!S!A1"])
        self.assertTrue(o.net_gain)
        self.assertIn("NOT ALLOWED", "\n".join(family_net.render(o)))

    def test_a_regression_past_the_thirteenth_digit(self) -> None:
        main = report([cell("A1", "residue13")], 100, 5)
        branch = report([cell("A1", "visible_numeric")], 102, 5)
        o = family_net.check(main, branch)
        self.assertFalse(o.ok)
        self.assertEqual([(r.before, r.after) for r in o.rejected], [("residue13", "visible_numeric")])

    def test_a_family_number_turning_into_an_error_is_rejected(self) -> None:
        o = family_net.check(report([], 100, 5), report([cell("A1", "visible_other")], 101, 5))
        self.assertEqual(len(o.rejected), 1)

    def test_no_net_gain(self) -> None:
        main = report([], 100, 5)
        for within15, visible in ((100, 5), (99, 5), (101, 6)):
            with self.subTest(within15=within15, visible=visible):
                o = family_net.check(main, report([cell("A1", "ulp1")], within15, visible))
                self.assertEqual(o.rejected, [])
                self.assertFalse(o.net_gain)
                self.assertFalse(o.ok)

    def test_a_text_check_that_starts_failing_is_rejected(self) -> None:
        text = [{"sheet": "S", "cell": "B1", "expected": "1", "actual": "2", "pass": False}]
        o = family_net.check(report([], 100, 5), report([], 101, 5, text=text))
        self.assertEqual([r.cell for r in o.rejected], ["c!S!B1"])

    def test_a_case_that_stops_running_is_rejected(self) -> None:
        o = family_net.check(report([], 100, 5), report([], 101, 5, checked=10, status="panic"))
        self.assertEqual([(r.cell, r.after) for r in o.rejected], [("c", "panic")])

    def test_a_case_with_new_coverage_is_not_compared(self) -> None:
        o = family_net.check(
            report([], 100, 5), report([cell("A1", "visible_numeric", fam=False)], 101, 5, checked=11)
        )
        self.assertEqual((o.regressions, o.compared_cases, o.uncompared_cases), ([], 0, 1))
        self.assertTrue(o.ok)

    def test_reports_that_cannot_be_compared(self) -> None:
        old = report([cell("A1", "ulp1")], 100, 5)
        del old["cases"][0]["cells"][0]["closeness"]
        unlisted = report([cell("A1", "ulp1")], 100, 5)
        unlisted["cases"][0]["passed"] = 10
        no_family = report([], 100, 5)
        del no_family["thirteen_digit_family"]
        other_suite = report([], 100, 5)
        other_suite["suites"] = [{"name": "gen-m9"}]
        for name, bad in (
            ("pre-T-4159", old),
            ("counts", unlisted),
            ("family", no_family),
            ("suites", other_suite),
        ):
            with self.subTest(name), self.assertRaises(family_net.ReportError):
                family_net.check(bad, report([cell("A1", "ulp1")], 101, 5))

    def test_an_unsupported_cell_ranks_last(self) -> None:
        unsupported = {"sheet": "S", "cell": "A1", "verdict": "unsupported"}
        o = family_net.check(report([], 100, 5), report([unsupported], 101, 5))
        self.assertEqual([r.after for r in o.rejected], ["unsupported"])
        # From unsupported to any bucket is no regression.
        self.assertEqual(
            family_net.check(
                report([unsupported], 100, 5), report([cell("A1", "visible_other")], 101, 5)
            ).regressions,
            [],
        )

    def test_exit_codes(self) -> None:
        with tempfile.TemporaryDirectory() as d:
            paths = {}
            for name, r in (
                ("main", report([], 100, 5)),
                ("good", report([cell("A1", "residue13")], 101, 5)),
                ("bad", report([cell("A1", "visible_numeric")], 101, 5)),
            ):
                paths[name] = Path(d) / f"{name}.json"
                paths[name].write_text(json.dumps(r), encoding="utf-8")
            (Path(d) / "junk.json").write_text("{", encoding="utf-8")
            for branch, code in (("good", 0), ("bad", 1), ("junk", 2)):
                with (
                    self.subTest(branch),
                    contextlib.redirect_stdout(io.StringIO()),
                    contextlib.redirect_stderr(io.StringIO()),
                ):
                    self.assertEqual(
                        family_net.main([str(paths["main"]), str(Path(d) / f"{branch}.json")]), code
                    )


if __name__ == "__main__":
    unittest.main()
