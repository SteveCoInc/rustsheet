---
id: T-4355
title: "Probe a negative and a spilled kept subnormal saved to xlsx"
milestone: M11
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish what Excel saves for a negative kept subnormal and a spilled/array kept subnormal, then confirm or revert the spill/array flush (already inferred in T-2211) and extend to negatives if evidenced.

## Context
T-2211 flushes only `0 < v < MIN_POSITIVE` on save (evidenced: probes/subnormal-saved-as-zero). Unprobed: `=0-EXP(-745)` beside `=(0-EXP(-745))=0` and `=(0-EXP(-745))*2^1000` (not `-EXP(-745)`, which is +2^-1074: numbers.md "Unary minus of a tiny subnormal"); a spilled / CSE array element (`=EXP({-745,-708})`); a positive subnormal constant that reaches the file without passing through a load (set through the API, which keeps it; numbers.md "A subnormal cell constant reads as 0"): `saved_number` now saves it as 0, and Excel's save of a typed subnormal constant is unprobed (T-2211 fidelity review); desktop Excel's save (decision 0039). Code: crates/rustsheet-xlsx/src/export/worksheets.rs `saved_number`. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cases committed with oracle values; `saved_number` and numbers.md "Worksheet EXP (T-1931)" updated to the evidence; export tests for each case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Negative zero and other number spellings in the file.

## Outcome
Probe `probes/subnormal-saved-negative-and-array` (excel-graph 2026-11-07): a negative kept subnormal (`=0-EXP(-745)`, `=0-EXP(-720)`) and a spilled array kept subnormal are saved as `<v>0</v>`; the in-memory value is kept (`=0` companions FALSE, `*2^1000` recovers it). `saved_number` now flushes either sign (the spill/array arms are confirmed); export test and numbers.md updated. Desktop Excel's save stays an M14 question. The array-context comparison `(EXP({-745,-708}))=0` is TRUE in Excel (probe D1): filed as T-4766. Eval delta: no scored change; the new probe's off cell D1 is owned by T-4766. This supersedes draft PR #3390 and salvage task T-4672.
