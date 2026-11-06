---
id: T-4520
title: "Count decision 0188 R3 cells (math-over-Excel by design) apart from visibly-off cells in the headline, rate and exits"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A cell where the engine deliberately returns the 50-digit math instead of Excel's value under
decision 0188 R3 is counted in its own closeness bucket, `r3_math`, shown on its own line of the
report and the progress dashboard, and is not visibly off anywhere visibly off is counted (the
headline, report.json `closeness`/`attributed`, the Phase 1 rate and tiers, the probe rows and
exits, the visible-parity lock). One definition, as decision 0267's `residue13` bucket is.

## Context
Owner-approved on 2026-11-06 ("Yes, go ahead with all three steps",
https://claude.ai/code/session_017265sRitBHUeqGLLXeJqDz).

- Decision 0188 R3: where Excel is more than G = 1e-4 off the 50-digit math on evidenced points,
  the engine follows the math; `eval/accuracy/math-over-excel.tsv` marks those cells.
- R1(c) (`crates/rustsheet-eval/src/r1c.rs`, `R1c::R3Pass`) already counts such a cell as
  passing in the milestone probe rows, but `closeness::bucket` still puts it in
  `visible_numeric`, so the headline (tools/src/rustsheet_tools/progress_report), the Phase 1
  rate and tiers and the visible-parity lock count it as visibly off. Example: 66
  probes-numeric/t3367-nper-tiny-rate cells (NPER below rate 1E-14).
- Precedent: decision 0267 and SPEC-0030 (`residue13`, `Visible::is_off`,
  `Visible::is_off_at_fifteen_digits`).
- Gate A's envelope (`accuracy/envelope.rs`) handles R3 its own way and must not change.

## Acceptance criteria
- [ ] One test decides R3 math-over-Excel cells (`r1c::r3_math`, the test behind `R1c::R3Pass`):
      a numeric cell with an R3 mark, Excel more than G off the math and the engine within ε of
      the math. The runner's results carry it as `CellResult::r3_math`, and `closeness::bucket`
      reads the flag.
- [ ] `Closeness` gains `r3_math` (`#[serde(default)]`); `scored()` includes it and `visible()`
      does not; `Visible::is_off` is false for it, `Visible::is_off_at_fifteen_digits` true (Gate
      A unchanged). Type, error and text cells never qualify.
- [ ] Unit tests: marked and within ε → `r3_math`; marked but the engine off the math → still
      visibly off; unmarked → unchanged; an error cell → unchanged.
- [ ] report.md's "Visibly off" table and the dashboard (bucket table, closeness sentence,
      history, "Since the last run") show the bucket; tools tests read old reports without the key
      as 0.
- [ ] A decision record citing 0188 R3, R1(c), 0267/SPEC-0030 and the owner's approval; SPEC-0030
      (or a section) lists the consumers; eval-harness.md and quality-gates.md updated.
- [ ] Bit-level baselines and `make eval-check` regressions unchanged; probes-numeric visibly-off
      before/after measured in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Adding or changing R3 marks; Gate A's envelope; Gate B's own R3 handling.
