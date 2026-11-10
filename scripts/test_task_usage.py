"""Tests for scripts/task_usage.py and the task usage tiers in scripts/tasks.py: the `usage:` key,
title inference, foundational-first ordering at equal priority, `next`/`ready` output and
`new --usage` (SPEC-0037 design 8, decision 0355 item 7). Stdlib only; run by `make hygiene`."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import task_usage  # noqa: E402 (the sys.path line above must run first)
import tasks  # noqa: E402
from test_tasks import write_task  # noqa: E402
from test_tasks_next import run_main  # noqa: E402

# A made-up catalog: the tiers here are chosen to test the matching, not copied from Excel's usage.
FOUNDATIONAL = ["IF", "SUM", "AND", "NA", "T", "LOG", "VLOOKUP"]
EXPERT = ["SUMIF", "T.INV.2T", "N", "TRUE", "LOG10", "XIRR"]


def write_scope(root: Path) -> None:
    rows = [{"name": n, "usage": "foundational"} for n in FOUNDATIONAL]
    rows += [{"name": n, "usage": "expert"} for n in EXPERT]
    path = root / task_usage.SCOPE
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"functions": rows}))


class Fixture(unittest.TestCase):
    def setUp(self) -> None:
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        write_scope(self.root)

    def usage(self, title: str) -> str | None:
        return task_usage.title_usage(title, self.root)


class TitleInference(Fixture):
    def test_the_first_catalog_name_decides(self) -> None:
        self.assertEqual(self.usage("XIRR and VLOOKUP of a blank"), "expert")
        self.assertEqual(self.usage("VLOOKUP and XIRR of a blank"), "foundational")
        self.assertEqual(self.usage("Probe CHOOSE then XIRR, then SUM"), "expert")  # CHOOSE is not in the catalog
        self.assertIsNone(self.usage("Nightly fuzz failures open an issue"))

    def test_a_name_counts_before_a_paren_a_comma_a_space_or_the_end(self) -> None:
        for title in ("SUM(A1:A3) of text", "SUM, MAX", "SUM over text", "Lift SUM"):
            self.assertEqual(self.usage(title), "foundational", title)
        # Any other character after it, and the name is not counted (SPEC-0037 design 8).
        for title in ("SUM's carry", "SUM/MAX", "SUM: carry", "SUM.X", "SUM-based"):
            self.assertIsNone(self.usage(title), title)

    def test_a_name_inside_a_longer_name_or_word_does_not_count(self) -> None:
        self.assertEqual(self.usage("IFERROR of SUMIF"), "expert")  # not IF, not SUM
        self.assertEqual(self.usage("T.INV.2T tails"), "expert")  # not T
        self.assertEqual(self.usage("LOG10 of a subnormal"), "expert")  # not LOG
        for title in ("XSUM drift", "_SUM drift", "Top-SUM rows", "P!SUM is #NAME?", "#SUM, again"):
            self.assertIsNone(self.usage(title), title)

    def test_names_match_upper_case_only(self) -> None:
        self.assertIsNone(self.usage("Sum and if over a log"))
        self.assertEqual(self.usage("If SUM is empty"), "foundational")

    def test_a_formula_quoting_a_foundational_function_after_the_subject(self) -> None:
        # T-3061 and T-3242 shapes: the subject comes first, the formula quotes SUM or IF after it.
        self.assertEqual(self.usage("Student-t quantile: Excel's XIRR is SUM(x*(1/y-1)) of y"), "expert")
        self.assertEqual(self.usage("XIRR small-shape cdf: the IF(a,SUM(x)) residue"), "expert")
        # A name skipped by the short-name rule does not decide: the next one does.
        self.assertEqual(self.usage("Legacy N over XIRR"), "expert")

    def test_short_names_that_read_as_words_count_only_before_a_paren(self) -> None:
        # N, T, NA, TRUE and FALSE: a letter, a value or "not available" in a title.
        for title in ("Legacy N and T over CHOOSE", "VALUETOTEXT of a TRUE cell", "NA as a result", "Top-N"):
            self.assertIsNone(self.usage(title), title)
        self.assertEqual(self.usage("T(A1) of long text"), "foundational")
        self.assertEqual(self.usage("VSTACK(NA(),range)"), "foundational")
        self.assertEqual(self.usage("N(+B1:B3) and TRUE()"), "expert")
        # Other short names count as words: all-caps IF and AND in a title are the functions.
        self.assertEqual(self.usage("IF, CHOOSE or LET picks a reference"), "foundational")
        self.assertEqual(self.usage("Move AND onto the shared walker"), "foundational")

    def test_without_a_catalog_there_is_no_tier(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            self.assertIsNone(task_usage.title_usage("SUM over text", Path(tmp)))

    def test_the_catalog_is_read_once(self) -> None:
        self.assertEqual(self.usage("SUM over text"), "foundational")
        (self.root / task_usage.SCOPE).unlink()
        self.assertEqual(self.usage("SUM over text"), "foundational")

    def test_the_real_catalog(self) -> None:
        def real(title: str) -> str | None:
            return task_usage.title_usage(title, tasks.ROOT)

        self.assertEqual(real("VLOOKUP approximate match over text"), "foundational")
        self.assertEqual(real("XIRR with a single cash flow"), "expert")
        self.assertEqual(real("T.INV.2T tails"), "expert")
        self.assertEqual(real("Student-t quantile: Excel's T.INV is SQRT(df*(1/y-1)) of y"), "expert")
        self.assertIsNone(real("Python facade: Transaction.move_sheet reorders a sheet by name"))


class EffectiveUsage(Fixture):
    def test_the_key_then_the_title_then_none(self) -> None:
        def effective(**t: object) -> str | None:
            return task_usage.effective_usage(t, self.root)

        self.assertEqual(effective(title="VLOOKUP of text", usage="expert"), "expert")
        self.assertEqual(effective(title="Parser cleanup", usage="foundational"), "foundational")
        self.assertEqual(effective(title="VLOOKUP of text"), "foundational")
        self.assertEqual(effective(title="XIRR of text"), "expert")
        self.assertIsNone(effective(title="Parser cleanup"))

    def test_validate_checks_the_key(self) -> None:
        with mock.patch.object(tasks, "ROOT", self.root):
            parsed = [
                tasks.parse(write_task(self.root, "T-0001", usage="foundational")),
                tasks.parse(write_task(self.root, "T-0002", usage="expert")),
                tasks.parse(write_task(self.root, "T-0003", usage="common")),
            ]
            errors = tasks.validate(parsed)
        self.assertEqual(len(errors), 1)
        self.assertIn("T-0003", errors[0])
        self.assertIn("usage='common'", errors[0])


class Ordering(Fixture):
    def ready(self, tier: str = "opus") -> list[str]:
        _, lines = run_main(self.root, ["tasks.py", "ready", "--tier", tier], (set(), set()), frozenset())
        return lines

    def test_foundational_first_at_equal_milestone_and_priority(self) -> None:
        write_task(self.root, "T-0001", title="XIRR of a blank")
        write_task(self.root, "T-0002", title="Parser cleanup")
        write_task(self.root, "T-0003", title="VLOOKUP of a blank")
        write_task(self.root, "T-0004", title="Parser cleanup", usage="foundational")
        self.assertEqual(
            [line.split()[0] for line in self.ready()[:-1]], ["T-0003", "T-0004", "T-0001", "T-0002"]
        )

    def test_priority_and_milestone_still_come_before_the_tier(self) -> None:
        write_task(self.root, "T-0001", title="VLOOKUP of a blank", priority="P2")
        write_task(self.root, "T-0002", title="XIRR of a blank", priority="P1")
        write_task(self.root, "T-0201", title="SUM of a blank", milestone="M2", priority="P1")
        write_task(self.root, "T-0003", title="XIRR of text", priority="P0", milestone="M2")
        self.assertEqual(
            [line.split()[0] for line in self.ready()[:-1]], ["T-0003", "T-0002", "T-0001", "T-0201"]
        )

    def test_next_and_ready_show_a_foundational_task(self) -> None:
        write_task(self.root, "T-0001", title="VLOOKUP of a blank")
        write_task(self.root, "T-0002", title="XIRR of a blank")
        lines = self.ready()
        self.assertEqual(lines[0], "T-0001  harness   VLOOKUP of a blank  usage: foundational")
        self.assertEqual(lines[1], "T-0002  harness   XIRR of a blank")
        _, lines = run_main(self.root, ["tasks.py", "next", "--tier", "opus"], (set(), set()), frozenset())
        self.assertEqual(
            lines, ["T-0001  VLOOKUP of a blank  (docs/tasks/M0/T-0001-x.md)  usage: foundational"]
        )


class NewUsage(Fixture):
    def run_new(self, *argv: str) -> tuple[int, str]:
        with (
            mock.patch.object(tasks, "ROOT", self.root),
            mock.patch.object(tasks, "TASKS", self.root / "docs/tasks"),
            mock.patch.object(
                tasks.sys, "argv", ["tasks.py", "new", "--milestone", "M0", "--lane", "harness", *argv]
            ),
            mock.patch.object(tasks, "git_task_ids", return_value=set()),
            mock.patch("builtins.print"),
        ):
            code = tasks.new_task([], fetch=False)
        files = list((self.root / "docs/tasks").rglob("*.md"))
        return code, files[0].read_text() if files else ""

    def test_the_flag_wins_over_the_title(self) -> None:
        code, text = self.run_new("--title", "VLOOKUP of a blank", "--usage", "expert")
        self.assertEqual(code, 0)
        self.assertIn("\ntier: opus\nusage: expert\ndepends_on: []\n", text)
        self.assertEqual(tasks.front_matter(text, Path("t"))["usage"], "expert")

    def test_without_the_flag_the_title_tier_is_written(self) -> None:
        self.assertIn("\nusage: foundational\n", self.run_new("--title", "VLOOKUP of a blank")[1])

    def test_a_title_without_a_tier_writes_no_key(self) -> None:
        code, text = self.run_new("--title", "Parser cleanup")
        self.assertEqual(code, 0)
        self.assertNotIn("usage:", text)

    def test_a_bad_value_is_refused(self) -> None:
        self.assertEqual(self.run_new("--title", "Parser cleanup", "--usage", "common"), (2, ""))


if __name__ == "__main__":
    unittest.main()
