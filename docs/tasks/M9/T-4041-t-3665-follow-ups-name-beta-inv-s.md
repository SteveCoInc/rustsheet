---
id: T-4041
title: "T-3665 follow-ups: name BETA.INV's search on bup+bgrat at a0 < 1, state the residual near a = 1, narrow T-3592's x29 cells"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The docs and owner lists match what T-3665 (PR #3098) changed: every caller of the new bup + bgrat path at a0 < 1 is
named, the residual near a = 1 is stated, and T-3592 lists only the x29 cells still off Excel's bits.

## Context
These are the MINOR findings of PR #3098's reviewers (excel-fidelity and design), given after every reviewer had
approved, so they were filed here rather than pushed (work-loop step 5).
- `bratio/mod.rs` `ported_tails_with`: the `Branch::BupBgrat if half_bgrat` arm (T-3665) also runs in BETA.INV's
  search (`beta_search_tails` passes `half_bgrat = true`), not only in BETA.DIST's cdf and F's tails. T-3665's Outcome
  and its statistics.md bullet name only those two. `beta_search_tails`'s doc still says branches that are not ported
  fall back to the bounded Cephes `search_tails`. No BETA.INV cell changed in the gated suites or in probes-numeric.
- docs/excel-semantics/statistics.md, "Incomplete beta is TOMS 708 `bratio`", the T-3665 bullet: "near the math"
  holds for the x29 region only. In probes-numeric/t2464-w2f-unit-shape (a just below 1, x = 0.1, b = 2.5 or 5), the
  new values are up to 29 ulp from the math, where the old ones were up to 7. Excel itself is up to 26 ulp off the
  math there and the new values track it: 6 cells are newly within 15 digits and several are bit-exact.
- docs/tasks/M9/T-3592-owners-for-121-unowned-last-digit-stat-cells.md: its split note says it drops the T-3665
  cells when T-3665 lands, but it still lists all 9. Only A1616 and A2393 of t2464-r2-bratio-x29 are still off
  Excel's bits, both within 15 digits. T-3665 also left 53 cells that are no longer bit-exact (52 in
  t2464-r2-bratio-x29, 1 in t2464-w2f-unit-shape; all within 15 digits) with no last-digit owner.
- The `ported_tails_with` doc-comment line that T-3665 extended runs over 100 columns: rewrap it.

## Acceptance criteria
- [ ] T-3665's statistics.md bullet and the `beta_search_tails` and `ported_tails_with` docs name BETA.INV's search as a
      caller of the bup + bgrat arm at a0 < 1.
- [ ] The bullet states the measured residual against the math in t2464-w2f-unit-shape, with case and cell examples.
- [ ] T-3592's t2464-r2-bratio-x29 owned-cells line lists only the cells still off Excel's bits (A1616, A2393, plus
      the 53 newly off-bit cells if T-3592 takes them as Phase 1b owners). `scripts/exit_owners.py --milestone M9`
      shows no new unowned cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to engine behavior (docs and owner lists only).

## Outcome
Docs and owner lists only; no engine behavior changed (no eval delta).
- `beta_search_tails` and `ported_tails_with` docs, and the T-3665 bullet in statistics.md, name BETA.INV's search as a
  caller of the bup + bgrat arm at a0 < 1; the doc-comment line is rewrapped under 100 columns.
- The bullet states the residual in t2464-w2f-unit-shape (up to 29 ulp from the math, old up to 7; Excel up to 26 ulp).
- T-3592's t2464-r2-bratio-x29 line now lists only A1616 and A2393. `scripts/exit_owners.py --milestone M9` reports the
  same 80 unowned cells before and after (none from these suites). The 53 cells no longer bit-exact are all within
  15 digits, so they are not exit cells and need no Phase 1b owner; T-3592 does not take them.
