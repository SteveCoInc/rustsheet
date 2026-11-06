---
id: T-4520
title: "Count decision 0188 R3 cells (math-over-Excel by design) apart from visibly-off cells in the headline, rate and exits"
milestone: M0
lane: harness
status: done
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
- [x] One test decides R3 math-over-Excel cells (`r1c::r3_math`, the test behind `R1c::R3Pass`):
      a numeric cell with an R3 mark, Excel more than G off the math and the engine within ε of
      the math. The runner's results carry it as `CellResult::target` = `Target::Math`, and `closeness::bucket`
      reads it.
- [x] `Closeness` gains `r3_math` (`#[serde(default)]`); `scored()` includes it and `visible()`
      does not; `Visible::is_off` is false for it, `Visible::is_off_at_fifteen_digits` true (Gate
      A unchanged). Type, error and text cells never qualify.
- [x] Unit tests: marked and within ε → `r3_math`; marked but the engine off the math → still
      visibly off; unmarked → unchanged; an error cell → unchanged.
- [x] report.md's "Visibly off" table and the dashboard (bucket table, closeness sentence,
      history, "Since the last run") show the bucket; tools tests read old reports without the key
      as 0.
- [x] A decision record citing 0188 R3, R1(c), 0267/SPEC-0030 and the owner's approval; SPEC-0030
      (or a section) lists the consumers; eval-harness.md and quality-gates.md updated.
- [x] Bit-level baselines and `make eval-check` regressions unchanged; probes-numeric visibly-off
      before/after measured in the Outcome.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Adding or changing R3 marks; Gate A's envelope; Gate B's own R3 handling.

## Outcome
- Decision 0310 records the rule. SPEC-0030 gains a section, "R3 math-over-Excel cells", that lists the consumers.
  eval-harness.md and quality-gates.md say where visibly off now stops.
- `r1c::r3_math` is the one R3 test: both values numbers, an R3 mark, Excel more than G off the math, and the engine
  inside Gate A's envelope with Excel's side dropped. `r1c::classify` calls it for `R3Pass`, and
  `r1c::mark_r3_math` stores it once per run as `CellResult::target` = `Target::Math`. The marking runs in `run` and
  `check`, before `RunReport::build`. `target` is an enum, not a bool: clippy's `struct_excessive_bools` does not
  allow a fourth bool.
- `closeness::bucket` gains `R3Math` and `Closeness` gains `r3_math`, with `#[serde(default)]`:
  - `scored()` includes it and `visible()` does not;
  - `Visible::is_off` is false for it and `Visible::is_off_at_fifteen_digits` is true, so Gate A is unchanged;
  - `Tier::of` places such a function no higher than `rate999`.
- The probe rows count these cells in `r3_pass`, not `visibly_off`, so `r1c_off` is unchanged. The R1(c) line,
  `r1c_left_out` and Gate B still classify them as `R3Pass`.
- report.md's "Visibly off" table gains an `r3_math` column. The dashboard shows the bucket in the bucket table, the
  closeness sentence, the charts (teal), history `off_cells.r3_math` and a "Since the last run" line. Each reads 0
  from an older report or row.
- Tests:
  - closeness: the bucket, the 15-digit stay, a type cell, and an old report.json;
  - r1c: marked and within ε gives `r3_math` and R3Pass; marked but the engine off the math, or Excel below G, stays
    visibly off; unmarked is unchanged; an error is unchanged;
  - attribution: the tier;
  - tools: an old closeness without the key, a history row without it, and the bucket line and sentence.
- **Measured** with `rustsheet-eval run --suite probes-numeric` at origin/main e72ae138, before and after:

  | measure | before | after |
  |---|---|---|
  | `visible_numeric` | 4,346 | 4,280 |
  | `r3_math` | 0 | 66 |
  | visibly off (`v15_checkpoint.visibly_off`) | 4,820 | 4,754 |
  | `attributed_off` | 4,647 | 4,581 |
  | `unattributed_off` | 173 | 173 |
  | R1(c) counts | 1,636 | 1,636 |
  | R1(c) Phase 1b | 3,118 | 3,118 |
  | R1(c) `r3_pass` | 66 | 66 |

  All 66 are probes-numeric/t3367-nper-tiny-rate cells. Pass, ulp, digits15, residue13 and visible_other are
  unchanged.
- Checks: `make check-fast` is green; it ran eval-gates (check --evidence-floor and accuracy) and py-check.
  `make eval-check` is green with no regressions, and visible parity is 1,970 against an allowance of 2,332. The
  bit-level baselines did not change. Gate A shows 24,642 points, 0 failing; its points never carry the mark.
- Follow-up: none needed. The steward's next `--lower-visible-lock` lowers whatever lines these cells free.
