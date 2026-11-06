---
id: T-4501
title: "PERMUTATIONA is #NUM! at a number_chosen of 2^31-1 or more"
milestone: M9
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
PERMUTATIONA matches Excel at a huge number_chosen (`#NUM!`) and gives Excel's own n^k, binary powering in doubles,
so the three Gate A known failures of accuracy/PERMUTATIONA/s1 go and its cells are bit-exact.

## Context
- statistics.md "PERMUTATIONA is binary powering in doubles, and #NUM! at a huge number_chosen (T-4106)": found by the
  T-4106 Gate A table.
- `PERMUTATIONA(1,2147483647)`, `PERMUTATIONA(1,1E10)` and `PERMUTATIONA(0,1E10)` are `#NUM!` in Excel (accuracy/
  PERMUTATIONA/s1 A76, A79, A71); the engine returns 1 and 0. `PERMUTATIONA(0,1000)` is 0 (A72). The cut lies in
  (1000, 2^31-1]: probe it (excel-probe) before choosing; 2^31-1 is PERMUT's number limit (T-1290), the likely one.
- Excel's value is right-to-left binary powering in doubles (r = 1, b = n; if k odd r *= b; b *= b; k /= 2): 104 of
  the 105 numeric s1 cells, against 73 for `powf` (the engine). The exception, `PERMUTATIONA(10,308)` = 1E+308 exactly
  (A68), is unexplained; find the rule (a probe of 10^k and other powers near the overflow) or keep it as a known miss.
- Code: `permutationa_of` in `crates/rustsheet-engine/src/functions/statistical/permut.rs`.
- Known failures: `eval/accuracy/known-failures.tsv` lines owned by T-4501 (remove them when they pass).

## Acceptance criteria
- [x] The number_chosen cut is probed and recorded in the semantics entry, and the engine is `#NUM!` from it on.
- [ ] PERMUTATIONA is binary powering in doubles; `rustsheet-eval accuracy` shows PERMUTATIONA/s1 bit-exact except any
      documented miss, and the T-4501 known failures are gone.
- [ ] `make check` green; reviewers approve.

## Out of scope
- PERMUTATIONA's number cut (about 1E+15 is `#NUM!`; T-2752 probes it).

## Outcome
Probe probes/t4501-permutationa-chosen-cut (36 cells, excel-graph 2026-11-06): the cut is exactly 2^31 - 1 on the
truncated number_chosen, at numbers 1 and 0 (2147483646.9 is a number; 2147483647, 2^31, 2^32 and 1E10 are `#NUM!`).
`permutationa_of` (permut.rs) is `#NUM!` from it, reusing PERMUT's `COUNT_LIMIT_EXCLUSIVE`; a unit test covers both
sides. Gate A: accuracy/PERMUTATIONA/s1 A71, A76 and A79 pass, and their three known-failures lines are gone
(`rustsheet-eval accuracy`: 0 failing). statistics.md records the cut as VERIFIED.
Not done here, on purpose: criterion 2, binary powering. It was chosen by scoring candidates against the s1 cells,
so it is a fit under decision 0188 R4 and owes a hold-out and a fresh sweep; every one of those cells is already inside
Gate A's envelope, so it is Phase 1b work, split to T-4557 (`defer: phase-1b`).
