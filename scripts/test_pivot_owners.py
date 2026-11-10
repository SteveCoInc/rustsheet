"""Tests for scripts/pivot_owners.py (T-4952)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import pivot_owners  # noqa: E402 (the sys.path line above must run first)
import tasks  # noqa: E402


class StalePivotOwners(unittest.TestCase):
    TASKS = [
        {"id": "T-0001", "status": "todo", "milestone": "M16"},
        {"id": "T-0002", "status": "done", "milestone": "M16"},
        {"id": "T-0003", "status": "in-progress", "milestone": "M14"},
    ]
    HEADER = "id\tstage\tevidence\towner\n"

    def test_open_m16_owners_and_empty_ones_pass(self) -> None:
        catalog = self.HEADER + "PV-0001\tPV1\t\tT-0001\nPV-0002\tPV1\t\t\n"
        self.assertEqual(pivot_owners.stale(self.TASKS, catalog, tasks.OPEN), [])

    def test_a_done_missing_or_other_milestone_owner_fails_with_its_line(self) -> None:
        catalog = self.HEADER + "PV-0001\tPV1\t\tT-0002\nPV-0002\tPV1\t\tT-9999\nPV-0003\tPV2\t\tT-0003\n"
        errors = pivot_owners.stale(self.TASKS, catalog, tasks.OPEN)
        self.assertEqual(len(errors), 3)
        self.assertIn(":2: PV-0001 owner T-0002 names a task that is done", errors[0])
        self.assertIn(":3: PV-0002 owner T-9999 names a task that does not exist", errors[1])
        self.assertIn(":4: PV-0003 owner T-0003 names a task that is M14's", errors[2])

    def test_no_catalog_or_no_owner_column_checks_nothing(self) -> None:
        self.assertEqual(pivot_owners.stale(self.TASKS, "", tasks.OPEN), [])
        self.assertEqual(pivot_owners.stale(self.TASKS, "id\tstage\nPV-0001\tT-0002\n", tasks.OPEN), [])

    def test_the_committed_catalog_names_only_open_m16_owners(self) -> None:
        self.assertEqual(pivot_owners.stale(tasks.load(), pivot_owners.catalog(tasks.ROOT), tasks.OPEN), [])


if __name__ == "__main__":
    unittest.main()
