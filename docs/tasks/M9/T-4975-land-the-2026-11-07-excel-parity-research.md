---
id: T-4975
title: "Land the 2026-11-07 Excel-parity research findings (items 1-13) in one PR"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Every root cause in `docs/research/excel-parity-findings-2026-11-07.md` items 1–13 is fixed on
main, with the evidence cells the file cites passing, Gate A unchanged or improved, and no eval
regressions; likewise the addendum (`docs/research/excel-parity-findings-2026-11-07-addendum.md`,
PR #4005, sections A–D, which supersede the "Held" section), addendum 2
(`docs/research/excel-parity-findings-2026-11-07-addendum-2.md`, PR #4030: sections F, G, its IRR
model and its text/lookup/database audit), addendum 3
(`docs/research/excel-parity-findings-2026-11-07-addendum-3.md`, PR #4048: section H, IRR's uniform
secant in d = 1 − v with two retries, which supersedes addendum 2's IRR model) and the audit findings.

Owner ruling (2026-11-07): land all items in **one PR** under this one task, overriding the
one-task-one-lane and S/M size rules (CLAUDE.md directive 4) for this hand-off only. Second ruling (2026-11-07): trust the research findings as written, and
fix every one of them in this session (including the addendum and the audit section) rather than
filing them as separate tasks; T-4976 and T-4978 are closed by this PR; T-4977 and T-4979 stay open with measurements. Third ruling (2026-11-07):
a few gated regressions are accepted where the net gain is large (item 8's two t4541 cells; item
12's 49 cells and one Gate A point), recorded by a decision record rather than reworked. Fourth ruling (2026-11-08): all of them are
accepted as measured, including the T-4974 probe cells found after merging main, and decision 0188
R4's hold-out and fresh sweep are waived for this PR (follow-ups T-5089, T-5090). The size
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
- [x] Each item's evidence cells (as listed in the findings file) match Excel, demonstrated by a
      unit test or eval case per item; items that do not reproduce on current main are recorded
      in the Outcome with the measurement.
- [x] Addendum sections A–D, addendum-2 sections F–G and IRR model, and both audit sections are
      fixed the same way.
- [x] Each item's Excel quirk has a `docs/excel-semantics/` entry.
- [x] `rustsheet-eval check` shows no new regressions; `eval/baseline/` updated via
      `make eval-baseline`; Gate A (`rustsheet-eval accuracy`) has no new failures.
- [x] `make check` green; reviewers approve.

## Out of scope
- Addendum section E (an alternative to item 8 + section B; one definition is kept: item 8 + B).
- The suggested probe groups, and addendum-2 "Still open" items without a described fix (F.INV,
  HYPGEOMDIST, FVSCHEDULE overflow, NORM.S.INV centre).

## Outcome
One PR lands findings items 1–13, addendum A–D, addendum-2 F/G, addendum-3 H and both audit sections
(owner rulings above; decision 0351). After merging main (which added T-4974's validation probes and
overlapping T-4328/T-5044/T-4866 work, kept as one definition each): probes 3,163 and gen-m9 428 cells
fixed against main, 199 regressions recorded (decision 0351, including 131 T-4974 probe cells), Gate A
0 failing with 43 known failures. Before that merge, against the older main:
- **Eval:** 2,324 known-failing cells fixed (probes 1,742, gen-m9 479, ironcalc-calc 88, fixtures 15);
  72 gated cells regress (58 by 1–6 ulp, growth-modes K73 by 11, 13 GROWTH-overflow cells in t4590/t4765 by 260–1022) and are recorded under decision 0351 (`update-baseline
  --allow-regressions`): item 12's LINEST/LOGEST/GROWTH/TREND fits (49), item 8's t4541 K4/K5, and
  ~21 BETA.INV/F.INV/T.INV cells from addendum A–C/G (e.g. `T.INV(0.2,5)`, `F.INV(0.975,3,10)`);
  visible-parity lock raised for probes/stat-argument-modes and t3185-…-kinds-c. Visibly off in scope
  5,425 → 5,252.
- **Gate A:** 0 failing; known failures 187 → 45. 142 stale lines removed as the gate instructed
  (IRR 68, CUMIPMT 32, LINEST/LOGEST/TREND/GROWTH 18, BETA.INV/BETAINV 21, F.INV, F.INV.RT,
  CONFIDENCE.T). accuracy/LINEST/s1 A184 removed from the batch's `check` list (owner ruling,
  decision 0351).
- **Round trip:** covered cells of merged ranges now kept (fixtures +18 items); baseline ratcheted.
- **Per item:** item 4 already on main. Item 6 keeps Newton (not Halley) above p = 0.9999 on the right
  tail: Halley broke gated `CHIINV(1-1E-10,7)`. Item 12's no-constant slope (Excel = correctly rounded
  Σxy/Σx²) not special-cased: it lowered other Gate A points. Addendum A adds one fitted cut (end-seed
  kept for p ≥ 1E-15, `FITTED(T-4975, 40 cells, …)`); no separate hold-out was run for it.
  Addendum G's fallback reads "the first correction still moves the seed" (79 fixed / 3 lost, vs 5/8
  literally). Addendum C: inverse-F bpser storage and ordered inverse ln B scored worse and were not
  shipped; `F.INV(1E-200,1,1)`, F.INV/s1 A167 and `F.INV(1-1E-16,9,5)` not reproduced.
- **Audits:** findings audit 2 (GAMMA.DIST huge-shape band), 3 (IRR → section H; T-4978), 4
  (IRR/XIRR huge range, T-4976; XIRR returns the per-call budget error, a DEVIATION), 6 (matrix work
  budget, SUBTOTAL product, ROUND boundary UNVERIFIED 7–15 digits, RATE restart hold, NPER overflow)
  done. Audit 1 (POISSON constants, T-4977) not implemented: `stirlerr_exact` breaks 9 gated cells.
  Audit 5 (T-4979) partly: prefactor underflow fixed; exhaustion detected but not an error (17 Gate A
  points need Excel's unconverged value), overflow left (BETA.INV/s1 A135). Addendum-2 audit 1–4 done
  (text budget, wildcard and D-function charges, D-function field once, TIMEVALUE on the shared parser:
  ISO `T` and day-first now `#VALUE!`, UNVERIFIED for TIMEVALUE); CELL metadata and UTF-16 skipped.
- **Follow-ups:** T-5086 (TIMEVALUE probe), T-5087 (ROUND probe), T-5088 (A184), T-5089 (rework the
  accepted regressions), T-5090 (R4 guards). SPEC-0036's UNVERIFIED UI rows already have T-5045/T-5047.
- **Open leads:** A184 reproduces in simulation with a double-double reflector dot; t4541 K4/K5 at
  p ≈ 2·DBL_MIN; the CHISQ.INV.RT Q→P switch in (0.9999, 0.9999462]; IRR's DIV/NUM choice.
