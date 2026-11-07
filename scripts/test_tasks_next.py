"""`next`/`ready` picking tests for `scripts/tasks.py`: tiers, handoffs, re-reviews, busy lanes per tier,
the functions-* sub-lanes and lane slots (decisions 0018, 0022, 0095, 0269, 0331). Split from
test_tasks.py by the file-size ratchet."""

from __future__ import annotations

import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import tasks  # noqa: E402 (the sys.path line above must run first)
from test_tasks import run_next, run_next_line, write_task  # noqa: E402


def run_main(
    root: Path, argv: list[str], claims: tuple[set[str], set[str]], active: frozenset[tuple[str, str, str, str]]
) -> tuple[int, list[str]]:
    """`tasks.main()`'s exit status and printed lines for `argv`, with these branch claims."""
    with (
        mock.patch.object(tasks, "ROOT", root),
        mock.patch.object(tasks, "TASKS", root / "docs/tasks"),
        mock.patch.object(tasks.sys, "argv", argv),
        mock.patch.object(tasks, "branch_claims", return_value=(*claims, set(active), {})),
        mock.patch("builtins.print") as printed,
    ):
        status = tasks.main()
    return status, [str(c[0][0]) for c in printed.call_args_list]


def run_ready(root: Path, tier: str, active: frozenset[tuple[str, str, str, str]]) -> list[str]:
    """The ids `ready` lists, then its "N ready" line."""
    _, lines = run_main(root, ["tasks.py", "ready", "--tier", tier], (set(), set()), active)
    return [line.split()[0] for line in lines[:-1]] + lines[-1:]


def run_next_exit(root: Path, tier: str, active: frozenset[tuple[str, str, str, str]], lane: str) -> int:
    return run_main(root, ["tasks.py", "next", "--tier", tier, "--lane", lane], (set(), set()), active)[0]


class Tiers(unittest.TestCase):
    def test_open_tasks_need_a_valid_tier_done_ones_do_not(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(write_task(root, "T-0001", tier=None)),
                    tasks.parse(write_task(root, "T-0002", tier="haiku")),
                    tasks.parse(write_task(root, "T-0003", status="done", tier=None)),
                    tasks.parse(write_task(root, "T-0004", tier="sonnet")),
                ]
                errors = tasks.validate(parsed)
        self.assertEqual(sum("tier=" in e for e in errors), 2)
        self.assertTrue(any("T-0001" in e for e in errors) and any("T-0002" in e for e in errors))

    def test_next_filters_by_tier_and_takes_handoffs_first(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus")
            write_task(root, "T-0002", tier="opus", body="\n## Handoff\nsonnet run stuck on X\n")
            write_task(root, "T-0003", tier="sonnet")
            self.assertEqual([run_next(root, "opus"), run_next(root, "sonnet")], ["T-0002", "T-0003"])

    def test_earlier_milestone_comes_before_priority_and_m9_still_gets_picked(self) -> None:
        # Decision 0026: an M2 function beats an M9 last-digit tail of higher priority.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0901", tier="opus", priority="P1", milestone="M9")
            write_task(root, "T-0210", tier="opus", priority="P2", milestone="M2")
            write_task(root, "T-0110", tier="opus", priority="P3", milestone="M10")
            self.assertEqual(run_next(root, "opus"), "T-0210")
            claims = ({"T-0210"}, set())
            self.assertEqual(run_next(root, "opus", claims=claims), "T-0901")

    def test_campaign_task_comes_before_milestone_order_but_after_p0_and_handoffs(self) -> None:
        # Decision 0083: a visible-parity M10 task beats an M2 task; P0 and handoffs still lead.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0210", tier="opus", priority="P1", milestone="M2")
            write_task(root, "T-1711", tier="opus", priority="P3", milestone="M10", campaign="visible")
            self.assertEqual(run_next(root, "opus"), "T-1711")
            write_task(root, "T-0002", tier="opus", priority="P2", body="\n## Handoff\nstuck\n")
            self.assertEqual(run_next(root, "opus"), "T-0002")
            write_task(root, "T-0001", tier="opus", priority="P0", milestone="M9")
            self.assertEqual(run_next(root, "opus"), "T-0001")

    def test_open_dependency_of_a_campaign_task_is_picked_as_campaign(self) -> None:
        # Decision 0094: a sweep the campaign fix waits on beats milestone order; a done one does not count.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0210", tier="opus", priority="P1", milestone="M2")
            write_task(
                root,
                "T-2145",
                tier="opus",
                priority="P3",
                milestone="M9",
                lane="research",
                depends_on="T-2144",
            )
            write_task(root, "T-2144", tier="opus", priority="P3", milestone="M9", lane="research")
            write_task(
                root,
                "T-2146",
                tier="opus",
                milestone="M9",
                lane="functions-stat",
                depends_on="T-2145",
                campaign="visible",
            )
            self.assertEqual(run_next(root, "opus"), "T-2144")
            write_task(
                root, "T-2144", tier="opus", priority="P3", milestone="M9", lane="research", status="done"
            )
            self.assertEqual(run_next(root, "opus"), "T-2145")
            write_task(
                root,
                "T-2146",
                tier="opus",
                milestone="M9",
                lane="functions-stat",
                depends_on="T-2145",
                campaign="visible",
                status="done",
            )
            self.assertEqual(run_next(root, "opus"), "T-0210")

    def test_campaign_stages_order_semantics_then_accuracy_then_visible(self) -> None:
        # Decision 0114: the stage beats milestone and priority among campaign tasks.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0210", tier="opus", priority="P0", milestone="M2", status="done")
            write_task(root, "T-0501", tier="opus", priority="P1", milestone="M5", campaign="visible")
            write_task(root, "T-0901", tier="opus", priority="P3", milestone="M9", campaign="accuracy")
            write_task(root, "T-0902", tier="opus", priority="P3", milestone="M9", campaign="semantics")
            self.assertEqual(run_next(root, "opus"), "T-0902")
            self.assertEqual(run_next(root, "opus", claims=({"T-0902"}, set())), "T-0901")
            self.assertEqual(run_next(root, "opus", claims=({"T-0902", "T-0901"}, set())), "T-0501")

    def test_dependency_takes_the_earliest_stage_that_needs_it(self) -> None:
        # Decision 0114: a sweep that a semantics task waits on comes before an accuracy task.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0901", tier="opus", milestone="M9", campaign="accuracy")
            write_task(root, "T-0950", tier="opus", milestone="M9", lane="research")
            write_task(
                root,
                "T-0951",
                tier="opus",
                milestone="M9",
                lane="functions-stat",
                depends_on="T-0950",
                campaign="visible",
            )
            write_task(
                root,
                "T-0952",
                tier="opus",
                milestone="M9",
                lane="functions-math",
                depends_on="T-0950",
                campaign="semantics",
            )
            self.assertEqual(run_next(root, "opus"), "T-0950")

    def test_deferred_task_is_never_ready_but_validates(self) -> None:
        # Decision 0114: `defer: phase-1b` parks last-digit work; an unknown deferral is rejected.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus", priority="P0", defer="phase-1b")
            write_task(root, "T-0002", tier="opus", priority="P3", milestone="M9")
            self.assertEqual(run_next(root, "opus"), "T-0002")
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(root / "docs/tasks/M0/T-0001-x.md"),
                    tasks.parse(write_task(root, "T-0003", defer="later")),
                ]
                errors = tasks.validate(parsed)
        self.assertTrue(any("defer=" in e and "T-0003" in e for e in errors))
        self.assertFalse(any("T-0001" in e for e in errors))

    def test_unknown_campaign_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(write_task(root, "T-0001", campaign="visible")),
                    tasks.parse(write_task(root, "T-0002", campaign="bits")),
                ]
                errors = tasks.validate(parsed)
        self.assertTrue(any("campaign=" in e and "T-0002" in e for e in errors))
        self.assertFalse(any("T-0001" in e for e in errors))

    def test_task_waiting_on_an_open_sweep_is_not_ready(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0901", tier="opus", milestone="M9", depends_on="T-0942")
            write_task(root, "T-0942", tier="opus", milestone="M9", lane="research")
            write_task(root, "T-0939", tier="opus", milestone="M9", priority="P2")
            self.assertEqual(run_next(root, "opus", lane="harness"), "T-0939")
            write_task(root, "T-0942", tier="opus", milestone="M9", lane="research", status="done")
            self.assertEqual(run_next(root, "opus", lane="harness"), "T-0901")

    def test_p0_comes_before_a_handoff(self) -> None:
        # T-0931 (P0 stack overflow) once waited behind the T-0125 (P2) handoff.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus", priority="P2", body="\n## Handoff\nsonnet run stuck\n")
            write_task(root, "T-0002", tier="opus", priority="P0")
            write_task(root, "T-0003", tier="opus", priority="P1")
            self.assertEqual(run_next(root, "opus"), "T-0002")

    def test_handoff_seen_only_on_its_branch_moves_the_task_to_opus(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus")
            write_task(root, "T-0002", tier="sonnet")  # main still says sonnet
            write_task(root, "T-0003", tier="sonnet")
            claims = ({"T-0002"}, {"T-0002"})  # pushed branch with a Handoff
            self.assertEqual(run_next(root, "opus", claims), "T-0002")
            self.assertEqual(run_next(root, "sonnet", claims), "T-0003")

    def test_resumed_handoff_is_no_longer_a_handoff(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus")
            write_task(root, "T-0002", tier="opus", body="\n## Handoff (resumed)\nopus run picked it up\n")
            # Resumed: its fresh branch is a claim again, so the next opus run skips it.
            self.assertEqual(run_next(root, "opus", ({"T-0002"}, set())), "T-0001")

    def test_claimed_task_is_skipped(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet")
            write_task(root, "T-0002", tier="sonnet")
            self.assertEqual(run_next(root, "sonnet", ({"T-0001"}, set())), "T-0002")

    def test_review_limited_task_is_resumed_by_opus_after_handoffs_and_before_new_work(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus", campaign="visible")
            write_task(root, "T-0002", tier="sonnet", lane="engine")  # blocked on its branch
            write_task(root, "T-0003", tier="opus", body="\n## Handoff\nstuck\n")
            write_task(root, "T-0004", tier="sonnet")
            claims = ({"T-0002"}, set())
            limited = {"T-0002": "agent/engine/T-0002-x"}
            busy = frozenset({("engine", "T-0009", "agent/engine/T-0009-x", "")})
            self.assertEqual(run_next(root, "opus", claims, busy, limited=limited), "T-0003")
            (root / "docs/tasks/M0/T-0003-x.md").unlink()
            line = run_next_line(root, "opus", claims, busy, limited=limited)
            self.assertEqual(line.split()[0], "T-0002")
            self.assertIn("resume agent/engine/T-0002-x", line)
            # Sonnet never re-reviews it, and a plain blocked claim is not offered at all.
            self.assertEqual(run_next(root, "sonnet", claims, limited=limited), "T-0004")
            self.assertEqual(run_next(root, "opus", claims), "T-0001")

    def test_ready_lists_what_next_could_pick_and_counts_it(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet")
            write_task(root, "T-0002", tier="sonnet")  # claimed by a pushed branch
            write_task(root, "T-0003", tier="opus")
            write_task(root, "T-0004", tier="sonnet", status="done")
            with (
                mock.patch.object(tasks, "ROOT", root),
                mock.patch.object(tasks, "TASKS", root / "docs/tasks"),
                mock.patch.object(tasks.sys, "argv", ["tasks.py", "ready", "--tier", "sonnet"]),
                mock.patch.object(tasks, "branch_claims", return_value=({"T-0002"}, set(), set(), {})),
                mock.patch("builtins.print") as printed,
            ):
                self.assertEqual(tasks.main(), 0)
        lines = [str(c[0][0]) for c in printed.call_args_list]
        self.assertEqual([lines[0].split()[0], lines[-1]], ["T-0001", "1 ready"])


class FunctionLanes(unittest.TestCase):
    def test_busy_lane_is_skipped_and_functions_means_any_sub_lane(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet", lane="functions-text")
            write_task(root, "T-0002", tier="sonnet", lane="functions-math")
            write_task(root, "T-0003", tier="sonnet", lane="engine")
            self.assertEqual(run_next(root, "sonnet", lane="functions"), "T-0001")
            busy_text = frozenset({("functions-text", "T-0009", "agent/functions-text/T-0009-x", "")})  # T-0009: open task on another branch
            self.assertEqual(run_next(root, "sonnet", lane="functions", active=busy_text), "T-0002")
            self.assertEqual(run_next(root, "sonnet", lane="functions-math"), "T-0002")

    def test_engine_takes_up_to_three_tasks_in_flight_and_other_lanes_one(self) -> None:
        # Decision 0269: most Phase 1 exit blockers are engine tasks, so engine has three slots.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for tid in ("T-0001", "T-0002", "T-0003", "T-0004"):
                write_task(
                    root, tid, tier="opus", lane="engine", status="in-progress" if tid < "T-0004" else "todo"
                )
            write_task(root, "T-0005", tier="opus", lane="functions-text")
            write_task(root, "T-0006", tier="opus", lane="functions-text", status="in-progress")
            two = frozenset({("engine", "T-0001", "agent/engine/T-0001-x", ""), ("engine", "T-0002", "agent/engine/T-0002-x", "")})
            self.assertEqual(run_next(root, "opus", active=two, lane="engine"), "T-0004")
            three = two | {("engine", "T-0003", "agent/engine/T-0003-x", "")}
            self.assertEqual(run_next(root, "opus", active=three, lane="functions"), "T-0005")
            self.assertEqual(
                run_next_line(root, "opus", active=three, lane="engine"),
                "lane engine busy: T-0001 in flight on agent/engine/T-0001-x",
            )
            # One active branch still fills a one-slot lane.
            one = frozenset({("functions-text", "T-0006", "agent/functions-text/T-0006-x", "")})
            self.assertEqual(
                run_next_line(root, "opus", active=one, lane="functions"),
                "lane functions-text busy: T-0006 in flight on agent/functions-text/T-0006-x",
            )

    def test_ready_p0_is_offered_in_a_busy_lane_and_a_p1_is_not(self) -> None:
        # Decision 0330: a P0 never waits for unrelated work in its lane.
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus", lane="harness", status="in-progress")
            write_task(root, "T-0002", tier="opus", lane="harness", priority="P1")
            write_task(root, "T-0003", tier="opus", lane="harness", priority="P0", milestone="M2")
            busy = frozenset({("harness", "T-0001", "agent/harness/T-0001-x", "")})
            self.assertEqual(run_next(root, "opus", active=busy, lane="harness"), "T-0003")
            self.assertEqual(run_ready(root, "opus", active=busy), ["T-0003", "1 ready"])
            # A P0 another worker has claimed is still skipped; the P1 behind the busy lane is reported.
            claims = ({"T-0003"}, set())
            self.assertEqual(
                run_next_line(root, "opus", claims, busy, lane="harness"),
                "lane harness busy: T-0001 in flight on agent/harness/T-0001-x",
            )

    def test_busy_lane_exits_3_and_an_empty_queue_exits_1(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="opus", lane="research", status="in-progress")
            write_task(root, "T-0002", tier="opus", lane="research")
            write_task(root, "T-0003", tier="sonnet", lane="engine")
            busy = frozenset({("research", "T-0001", "agent/research/T-0001-x", "")})
            self.assertEqual(run_next_exit(root, "opus", busy, "research"), 3)
            # No opus task in engine at all: an empty queue, not a busy lane.
            self.assertEqual(run_next_exit(root, "opus", busy, "engine"), 1)
            self.assertEqual(run_next_line(root, "opus", active=busy, lane="engine"), "no ready task")
            # A busy lane whose only ready task is of the other tier is empty for this tier.
            self.assertEqual(run_next_exit(root, "sonnet", busy, "research"), 1)

    def test_handoff_is_resumed_even_in_a_busy_lane(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet", lane="functions-stat", body="\n## Handoff\nstuck\n")
            self.assertEqual(
                run_next(root, "opus", active=frozenset({("functions-stat", "T-0008", "agent/functions-stat/T-0008-x", "")})), "T-0001"
            )

    def test_branch_of_a_done_task_does_not_keep_its_lane_busy(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet", lane="functions-text", status="done")
            write_task(root, "T-0002", tier="sonnet", lane="functions-text")
            # T-0001 merged a minute ago and its branch still exists: the lane is free again.
            self.assertEqual(
                run_next(root, "sonnet", active=frozenset({("functions-text", "T-0001", "agent/functions-text/T-0001-x", "")})), "T-0002"
            )

    def test_busy_lane_comes_from_the_tasks_own_lane_not_the_branch_name(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_task(root, "T-0001", tier="sonnet", lane="functions-text", status="in-progress")
            write_task(root, "T-0002", tier="sonnet", lane="functions-text")
            write_task(root, "T-0003", tier="sonnet", lane="functions-math")
            # A branch pushed as agent/functions/T-0001-... still makes functions-text busy.
            self.assertEqual(run_next(root, "sonnet", active=frozenset({("functions", "T-0001", "agent/functions/T-0001-x", "")})), "T-0003")

    def test_plain_functions_lane_is_retired_for_open_tasks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(tasks, "ROOT", root):
                parsed = [
                    tasks.parse(write_task(root, "T-0001", lane="functions")),
                    tasks.parse(write_task(root, "T-0002", status="done", lane="functions")),
                    tasks.parse(write_task(root, "T-0003", lane="functions-lookup")),
                ]
                errors = tasks.validate(parsed)
        self.assertEqual([e for e in errors if "retired" in e and "T-0001" in e], errors)
        self.assertEqual(len(errors), 1)


class BusyPerTier(unittest.TestCase):
    """A lane is busy for one tier at a time (decision 0331)."""

    @staticmethod
    def lane_with_both_tiers(root: Path) -> None:
        write_task(root, "T-0001", tier="sonnet", lane="engine", status="in-progress")
        write_task(root, "T-0002", tier="opus", lane="harness", status="in-progress")
        write_task(root, "T-0003", tier="opus", lane="harness")
        write_task(root, "T-0004", tier="sonnet", lane="harness")

    def test_a_sonnet_branch_hides_only_sonnet_tasks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.lane_with_both_tiers(root)
            write_task(root, "T-0005", tier="sonnet", lane="harness", status="in-progress")
            sonnet = frozenset({("harness", "T-0005", "agent/harness/T-0005-x", "sonnet")})
            self.assertEqual(run_next(root, "opus", active=sonnet, lane="harness"), "T-0003")
            self.assertEqual(
                run_next_line(root, "sonnet", active=sonnet, lane="harness"),
                "lane harness busy: T-0005 in flight on agent/harness/T-0005-x",
            )
            self.assertEqual(run_ready(root, "opus", active=sonnet), ["T-0003", "1 ready"])
            self.assertEqual(run_ready(root, "sonnet", active=sonnet), ["0 ready"])

    def test_an_opus_branch_hides_only_opus_tasks(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.lane_with_both_tiers(root)
            # The tier comes from main's task file when the branch's gives none.
            opus = frozenset({("harness", "T-0002", "agent/harness/T-0002-x", "")})
            self.assertEqual(run_next(root, "sonnet", active=opus, lane="harness"), "T-0004")
            self.assertEqual(
                run_next_line(root, "opus", active=opus, lane="harness"),
                "lane harness busy: T-0002 in flight on agent/harness/T-0002-x",
            )
            self.assertEqual(run_ready(root, "sonnet", active=opus), ["T-0004", "1 ready"])
            self.assertEqual(run_ready(root, "opus", active=opus), ["0 ready"])

    def test_a_resumed_handoff_counts_as_opus_and_a_branch_of_unknown_tier_as_both(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.lane_with_both_tiers(root)
            write_task(root, "T-0005", tier="sonnet", lane="harness", status="in-progress")
            # Main still says sonnet; branch_claims reports the branch's handoff as opus.
            resumed = frozenset({("harness", "T-0005", "agent/harness/T-0005-x", "opus")})
            self.assertEqual(run_next(root, "sonnet", active=resumed, lane="harness"), "T-0004")
            self.assertEqual(run_next_exit(root, "opus", resumed, "harness"), 3)
            # T-0009 is on no task file we can read: it takes a slot of both tiers.
            unknown = frozenset({("harness", "T-0009", "agent/harness/T-0009-x", "")})
            self.assertEqual(run_next_exit(root, "opus", unknown, "harness"), 3)
            self.assertEqual(run_next_exit(root, "sonnet", unknown, "harness"), 3)

    def test_a_lane_with_one_branch_of_each_tier_offers_nothing(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.lane_with_both_tiers(root)
            write_task(root, "T-0005", tier="sonnet", lane="harness", status="in-progress")
            both = frozenset(
                {
                    ("harness", "T-0002", "agent/harness/T-0002-x", "opus"),
                    ("harness", "T-0005", "agent/harness/T-0005-x", "sonnet"),
                }
            )
            for tier in ("opus", "sonnet"):
                self.assertEqual(run_next_exit(root, tier, both, "harness"), 3, tier)
            self.assertEqual(run_main(root, ["tasks.py", "next", "--lane", "harness"], (set(), set()), both)[0], 3)

    def test_branch_claims_takes_the_tier_from_the_branch_task_file(self) -> None:
        now = int(time.time())
        refs = "".join(
            f"refs/remotes/origin/agent/harness/{tid}-x {now - 60}\n" for tid in ("T-0001", "T-0002", "T-0003", "T-0004")
        )
        bodies = {
            "T-0001": "tier: sonnet\n---\n",
            "T-0002": "tier: sonnet\n---\n## Handoff (resumed)\nnotes\n",
            "T-0003": "tier: sonnet\n---\nResumed after the review-round limit (step 5c).\n",
            "T-0004": "tier: opus\n---\n",
        }

        def fake_run(cmd: list[str], **_: object) -> mock.Mock:
            args = cmd[3:]
            if args[0] == "for-each-ref":
                return mock.Mock(stdout=refs)
            if args[0] == "ls-tree":
                return mock.Mock(stdout=f"docs/tasks/M0/{args[3].rsplit('/', 1)[-1][:6]}-x.md\n")
            tid = args[1].rsplit("/", 1)[-1][:6]
            return mock.Mock(stdout=f"---\nid: {tid}\nstatus: in-progress\n{bodies[tid]}")

        with mock.patch.object(tasks.subprocess, "run", side_effect=fake_run):
            _, _, active, _ = tasks.branch_claims(fetch=False)
        self.assertEqual(
            {(tid, tier) for _, tid, _, tier in active},
            {("T-0001", "sonnet"), ("T-0002", "opus"), ("T-0003", "opus"), ("T-0004", "opus")},
        )


class CurrentMilestonePassesLaterWork(unittest.TestCase):
    """A current-milestone task passes a lane busy only with later-milestone work (decision 0332)."""

    MILESTONES = "# M\n\n## Phase order\n\n| Phase | Milestones, in order |\n|---|---|\n| 1 | M0, M1, M2, M5, M9 |\n| 2 | M3 |\n\n## Next\n"

    def board(self, root: Path, snapshot: bool = True) -> None:
        (root / "docs").mkdir(parents=True, exist_ok=True)
        (root / "docs/milestones.md").write_text(self.MILESTONES)
        heading = "## Snapshot — 2026-11-07 (M2 is current)" if snapshot else "## Snapshot — 2026-11-07"
        (root / "docs/status.md").write_text(f"# Status\n\n{heading}\n")
        write_task(root, "T-0001", lane="research", milestone="M9", status="in-progress")
        write_task(root, "T-0002", lane="research", milestone="M0", status="in-progress")
        write_task(root, "T-0003", lane="research", milestone="M2", status="in-progress")
        write_task(root, "T-0004", lane="research", milestone="M2")
        write_task(root, "T-0005", lane="research", milestone="M3")

    @staticmethod
    def flying(*tids: str) -> frozenset[tuple[str, str, str, str]]:
        return frozenset(("research", tid, f"agent/research/{tid}-x", "opus") for tid in tids)

    def test_a_later_milestone_branch_lets_only_the_current_milestone_task_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.board(root)
            later = self.flying("T-0001")
            self.assertEqual(run_next(root, "opus", active=later, lane="research"), "T-0004")
            self.assertEqual(run_ready(root, "opus", active=later), ["T-0004", "1 ready"])
            # M3 is later than M9 in the phase order, though its number is lower: it is not passed.
            self.assertEqual(tasks.milestone_order.read(root), ("M2", ["M0", "M1", "M2", "M5", "M9", "M3"]))

    def test_an_m0_or_current_milestone_branch_is_not_passed(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.board(root)
            for tid in ("T-0002", "T-0003"):
                self.assertEqual(
                    run_next_line(root, "opus", active=self.flying(tid), lane="research"),
                    f"lane research busy: {tid} in flight on agent/research/{tid}-x",
                )

    def test_no_snapshot_heading_turns_the_rule_off(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.board(root, snapshot=False)
            self.assertEqual(run_next_exit(root, "opus", self.flying("T-0001"), "research"), 3)
            self.assertEqual(run_ready(root, "opus", active=self.flying("T-0001")), ["0 ready"])

    def test_the_pass_is_taken_once_per_lane_and_tier(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.board(root)
            # T-0003 (M2) already passed T-0001 (M9): the lane holds a current-milestone task now.
            self.assertEqual(run_next_exit(root, "opus", self.flying("T-0001", "T-0003"), "research"), 3)
            # A branch whose task is not on main is of no known milestone: never passed.
            self.assertEqual(run_next_exit(root, "opus", self.flying("T-0009"), "research"), 3)


if __name__ == "__main__":
    unittest.main()
