---
id: T-4975
title: "Land the 2026-11-07 Excel-parity research findings (items 1-13) in one PR"
milestone: M9
lane: functions-stat
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Every root cause in `docs/research/excel-parity-findings-2026-11-07.md` items 1–13 is fixed on
main, with the evidence cells the file cites passing, Gate A unchanged or improved, and no eval
regressions. The held inverse-beta/F work stays out (it breaks Gate A).

Owner ruling (2026-11-07): land all items in **one PR** under this one task, overriding the
one-task-one-lane and S/M size rules (CLAUDE.md directive 4) for this hand-off only. The size
field says M because `scripts/tasks.py` refuses L; the real size is L.

## Context
The research run (PR #3998) prototyped 13 independent fixes on main `2e192ce48` and measured Gate B
462 → ~488 with Gate A clean. Upstream has moved, so each item is re-measured here. Item → existing
per-item task (the per-item tasks keep their history; this task's Outcome records which of them
this PR closes):

| # | Item | Per-item task | Code |
|---|---|---|---|
| 1 | Covered cells of a merged range keep their content | T-4722 | rustsheet-xlsx `import/merged_cells.rs`, engine paste/autofill |
| 2 | SKEW/SKEW.P/KURT stored x87 moments | T-4328 | `statistical/count_and_average.rs` |
| 3 | BINOM.DIST.RANGE Loader deviance store points | T-2146 | `statistical/loader.rs` |
| 4 | POISSON far upper tail, direct deviance | T-3469 | `incomplete_gamma/excel_tail.rs` (already on main; verify) |
| 5 | Gamma small-x prefactor from separately rounded factors | T-3219 | `incomplete_gamma/mod.rs`, `density.rs`, `gamma.rs` |
| 6 | CHIINV/CHISQ.INV/CHISQ.INV.RT split complement + Halley | T-3146 | `incomplete_gamma/closed_form.rs` |
| 7 | T.TEST/TTEST via the ported beta kernel | T-3730 | `statistical/student_t.rs` |
| 8 | T.INV small-beta inverse tail | T-3363 | `statistical/inverse_beta.rs` |
| 9 | T.DIST.2T rounded df-1 argument, large-df centre | T-2930 | `statistical/student_t.rs` / `t_dist` |
| 10 | LOGNORM.DIST density TwoSum chain | T-4321 | `statistical/log_normal.rs` |
| 11 | IRR discount domain validated at convergence | T-2928 | `financial/financial_util.rs` |
| 12 | LINEST/LOGEST centered pivoted Householder QR | T-2690 | `statistical/linest.rs` |
| 13 | CUMIPMT/CUMPRINC grouped balance drop | T-3731 | `financial/cumulative.rs` |

## Acceptance criteria
- [ ] Each item's evidence cells (as listed in the findings file) match Excel, demonstrated by a
      unit test or eval case per item; items that do not reproduce on current main are recorded
      in the Outcome with the measurement.
- [ ] Each item's Excel quirk has a `docs/excel-semantics/` entry.
- [ ] `rustsheet-eval check` shows no new regressions; `eval/baseline/` updated via
      `make eval-baseline`; Gate A (`rustsheet-eval accuracy`) has no new failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The "Held" inverse-beta/F section (BETA.INV, BETAINV, T.INV.2T, TINV, CONFIDENCE.T, FINV,
  F.INV.RT): fails Gate A as prototyped.
- The audit findings section (filed as follow-up tasks, not fixed here).
- The suggested probe groups.
