---
id: T-4078
title: "Iterative-calculation sweeps hash cell keys with the default hasher, 15% of a sweep's cost"
milestone: M12
lane: perf
status: done
priority: P1
size: S
tier: opus
depends_on: [T-4048]
---

## Goal
Iterative-calculation sweeps run faster with no change in results: the iteration state's maps use
the evaluation's position hasher (`KeyBuild`) instead of the default SipHash.

## Context
Found while investigating T-4048 (nightly `bitcode_workbook` fuzz timeouts). Excel-valid settings
still allow long sweeps. Take a 1,024-cell cycle that never converges (`A1=A1024+1`,
`A<n>=A<n-1>+1`), with iterateCount 32,767 (Excel's maximum) and iterateDelta 0. It runs the
whole `MAX_SWEEP_EVALUATIONS` budget, 2^25 member evaluations, which takes 8.25 s per
recalculation in a release build.

That is inside SPEC-0028's accepted worst case (the 2^32-unit ceiling, 100 to 125 s), and Excel
does the same sweeps. So the owner chose to speed the sweeps up and leave the cap and the ceiling
alone (no new DEVIATION).

Callgrind on a 16-cell version of the cycle (524k sweep evaluations) found about 15% of the
instructions in SipHash (`RandomState::hash_one::<(u32, i32, i32)>`). That hashing comes from
`IterationState::members`, `prior` and `settled` (iteration.rs): every read during a sweep checks
`is_cycle_member`. `Evaluation::cells` already uses `KeyBuild` for the same keys (formula
positions from the workbook), so these maps can share it.

## Acceptance criteria
- [x] `IterationState`'s maps use `KeyBuild`; no other behavior change.
- [x] Existing iteration and recalc-budget tests pass unchanged (results identical).
- [x] The 1,024-cell × 32,767 case is measurably faster in a release build.
- [x] T-4048 review MINOR: the importer test module's `allow(clippy::unwrap_used)` gives a reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Lowering `MAX_SWEEP_EVALUATIONS` or the SPEC-0028 ceiling (owner decision: speed only).
- Evaluator-wide costs left in the profile: `get_cell_value`, dropping `CalcResult`,
  `SheetData::cell`, the formula walk, each under about 7%.

## Outcome
**Change.** `members`, `prior` and `settled` are now `HashSet<Key, KeyBuild>` and
`HashMap<Key, Cell, KeyBuild>`. This adds no new exposure to chosen keys: `Evaluation::cells`
already hashes the same formula positions with `KeyBuild`.

**Measurements** (release build, 1,024-cell cycle, iterateCount 32,767, delta 0; A1 = 33,552,385
before and after):
- one recalculation: 8.25 s before, 7.0 s after;
- callgrind on the 16-cell cycle: 1.578e9 instructions before, 1.323e9 after (−16%), and SipHash
  is gone from the profile.

**What is left.** The remaining cost is the general evaluation path, about 2,500 instructions per
member evaluation, spread over cell reads, `CalcResult` construction and drop, and the formula
walk. No sweep-specific item is above about 4%.
