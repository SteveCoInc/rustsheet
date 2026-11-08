---
id: T-5005
title: "Round-trip R2 sheet-level scenarios: rename, add, delete and move a sheet (SPEC-0033 R2, SPEC-0034 D5.7)"
milestone: M15
lane: harness
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-4562]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
The round-trip suites score structural edits that need no M4: rename, add, delete and move a sheet. These are the
R2 scenarios of SPEC-0033, so the gains and drops of stage 5 are measured.

## Context
SPEC-0033 defines R2 (D1 table, the `stale` verdict, D7.2's rule that R2 drops explain themselves). It defers R2
scoring to M4 because of row and column edits. The sheet-level edits are already real `Txn` edits in the facade
(`crates/rustsheet/src/edit.rs`). The scenario table is `crates/rustsheet-eval/src/roundtrip/scenarios.rs`, and the
expected-delta rule is R1's, `roundtrip/r1.rs`.

## Acceptance criteria
- [x] Scenarios R2/rename, R2/add, R2/delete and R2/move exist, each with its expected delta: a renamed sheet's name
      items, a deleted sheet's parts reported `OwnerDeleted`.
- [x] The fixtures suite runs them, and the corpus suites run R2/rename and R2/delete. The baselines are recorded under
      a decision record if they add failing items.
- [x] SPEC-0033 is edited to say sheet-level R2 is scored now: the Non-goals bullet "R2 implementation", the D1
      table's R2 row (the milestone column), and Rollout item 5's "R2 scoring follows M4".
- [ ] `make check` green; reviewers approve. (`make check-fast` is green on 568d1e0b6; the review panel was started and stopped, so no verdict exists.)

## Out of scope
- Row and column R2 (after M4).

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).

## Handoff (work in progress, 2026-11-08)
The session working this task stopped on the owner's request before review. The work is on branch
`agent/harness/T-5005-r2-sheet-scenarios` (implementation, tests, SPEC-0033 and SPEC-0034 edits, decision 0352,
baselines recorded for fixtures, excel-saved, poi and corpus-bench, nightly timeout 600 s, follow-up T-5079, T-5077's
criteria updated). No PR is open. To finish it:
1. Claim it again (decision 0306), check out the branch, and merge origin/main. On a baseline conflict take main's
   version and re-record with `target/release/rustsheet-eval roundtrip --update-baseline --suite <suite> --timeout 300`,
   one suite at a time with `RAYON_NUM_THREADS=3`: all suites at once ran out of memory, and the pipe hid the kill.
   Do not record calipers: main's T-5077 regression makes the ratchet refuse it (decision 0352).
2. `make check-fast`, then the review panel (design, security, Excel fidelity: the diff touches eval/baseline).
3. Open the PR (repo template), enable auto-merge, watch CI. The gated poi and corpus-bench round trips take at most
   32 s a case locally, against CI's 60 s default timeout; watch that on CI's slower runners.

## Progress so far
- `crates/rustsheet-eval/src/roundtrip/r2.rs` holds the four sheet edits (`SheetEdit`) and their expected delta.
  Each acts on the first sheet in tab order. Rename and add use the name `rustsheet-roundtrip-R2`. Delete and move
  are `n/a` for a one-sheet workbook. `scenarios.rs` runs them as `Run::Sheet`, the rates line reports `R2`, and
  `r1::violations` now takes the run's explanation rule.
- A renamed sheet keeps its identity: the output's part keys and semantic facts are keyed back under the source's
  name (`Rename`), so only what the rename rewrote can differ. The engine does not report the formulas a rename or
  delete rewrites, so the harness compares every formula before and after.
- `dropped.rs` accepts `OwnerDeleted` for a deleted sheet's parts and `StaleAfterStructuralEdit` naming the edited
  sheet, in R2 runs only (SPEC-0033 D7.2). The report emits neither until T-5004, so a deleted sheet's worksheet part
  is a known `lost` item until then.
- First-record R2 kept rates: calipers 97.38%, corpus-bench 85.93%, poi 90.82%, excel-saved 85.24%, fixtures 81.39%.
  Every R2 save is regenerated, so these measure SPEC-0034 stage 5. The fixtures, excel-saved, poi and corpus-bench
  baselines took them as new coverage (decision 0352). Calipers' R2 keys wait for T-5077's re-record: main's R0
  regression of `pivot_named_range` makes the ratchet refuse calipers. The nightly's round-trip timeout rose to
  600 s, because calipers pv26_002 takes 287 s with the R2 scenarios.
- `R2/delete` found an engine bug: a deleted sheet's sheet-scoped names stay, and the save fails (78 corpus cases).
  It is filed as T-5079.
