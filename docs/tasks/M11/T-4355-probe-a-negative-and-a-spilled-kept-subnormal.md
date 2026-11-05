---
id: T-4355
title: "Probe a negative and a spilled kept subnormal saved to xlsx"
milestone: M11
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Establish what Excel saves for a negative kept subnormal and a spilled/array kept subnormal, then confirm or revert the spill/array flush (already inferred in T-2211) and extend to negatives if evidenced.

## Context
T-2211 flushes only `0 < v < MIN_POSITIVE` on save (evidenced: probes/subnormal-saved-as-zero). Unprobed: `=0-EXP(-745)` beside `=(0-EXP(-745))=0` and `=(0-EXP(-745))*2^1000` (not `-EXP(-745)`, which is +2^-1074: numbers.md "Unary minus of a tiny subnormal"); a spilled / CSE array element (`=EXP({-745,-708})`); desktop Excel's save (decision 0039). Code: crates/rustsheet-xlsx/src/export/worksheets.rs `saved_number`. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cases committed with oracle values; `saved_number` and numbers.md "Worksheet EXP (T-1931)" updated to the evidence; export tests for each case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Negative zero and other number spellings in the file.
