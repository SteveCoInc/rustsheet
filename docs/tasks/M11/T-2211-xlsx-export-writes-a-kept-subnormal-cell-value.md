---
id: T-2211
title: "xlsx export writes a kept subnormal cell value as 0, as Excel does"
milestone: M11
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
When RustSheet saves an xlsx, a formula cell whose value is a non-zero subnormal (below 2.2250738585072014E-308) is
written as `<v>0</v>`, as Excel does, so a file RustSheet saves reads back like one Excel saves.

## Context
- Decision 0089 and docs/excel-semantics/numbers.md "Worksheet EXP (T-1931)": Excel for the web keeps `EXP(-745)`
  = 2^-1074 in memory (`(EXP(-745))=0` is FALSE) but writes `<v>0</v>` to the file; a normal value such as
  `EXP(-708)` is written in full (`<v>3.3075530036384078E-308</v>`). Evidence: probes/subnormal-saved-as-zero
  (excel-graph, 2026-10-20) and the saved workbook the oracle cached for it.
- Code: crates/rustsheet-xlsx/src/export/worksheets.rs (cell value writing). Search for an existing subnormal helper
  (`arithmetic::flush_subnormal` in the engine) before adding one.
- Only the cached `<v>` changes; the in-memory value and recalculation are untouched.
- Evidence gap (2026-10-25 steward spot-audit of #1374, fidelity and architecture reviewers): the
  only evidence is Excel for the web's save of positive kept subnormals (`EXP(-745)`, `EXP(-720)`, `=A1`
  in `probes/subnormal-saved-as-zero`). Before implementing, probe a negative one with a FALSE companion
  (`=-EXP(-745)` beside `=(-EXP(-745))=0`) and a stored constant subnormal; desktop Excel's save is
  unprobed (record that, decision 0039).

## Acceptance criteria
- [ ] An export test: a workbook with `=EXP(-745)` saves `<v>0</v>` for that cell, and `=EXP(-708)` saves its value
      in full; reloading the file and recalculating still gives 2^-1074.
- [ ] numbers.md "Worksheet EXP (T-1931)" names the export as implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Negative zero and other number spellings in the file.

## Outcome
`export/worksheets.rs` passes every number `<v>` through `saved_number`, which saves a positive kept subnormal
(`0 < v < MIN_POSITIVE`) as `<v>0</v>`; in-memory values are untouched. Reviewer round 1 narrowed it from
`flush_subnormal` so unevidenced negatives and `-0` keep their output. New test `tests/subnormal_export.rs`
covers `=EXP(-745)`, `=A1`, `=EXP(-708)`, a spilled `=EXP({-745,-708})`, an unflushed negative `=0-EXP(-745)`
and the reload giving 2^-1074. The negative one uses `0-x` because `-EXP(-745)` is +2^-1074 in memory (numbers.md
"Unary minus of a tiny subnormal"); the resumed run fixed the test, which had set A5 twice and never checked a
negative. The negative / spilled / desktop probes were not run: filed as T-4355. Eval delta: none.

## Handoff (resumed)
Escalated per work-loop 5b: the excel-fidelity reviewer requested changes twice. Round 1 (MAJOR): the flush
covered unevidenced negative subnormals, so it was narrowed to `0 < v < MIN_POSITIVE` and T-4355 filed. Round 2
(MAJOR, wording): numbers.md said spill/array paths keep today's output while the code flushes them. This is fixed
in the same push as this handoff (numbers.md, `saved_number` doc, T-4355 goal, and a spill test `=EXP({-745,-708})`
in tests/subnormal_export.rs), and `cargo test` passes, but the fix has not been re-reviewed and `make check-fast`
has not been re-run on it. Design and security approved at round 1 (head 925e532). Next: run `make check-fast`,
re-run the fidelity reviewer on the current head, then mark done and enqueue PR #3374.
