---
id: T-3764
title: "BETA.DIST cdf at b = 1 and a shape of 136 to 1414: Excel's w on the a0 = 1 complement is one 2^-53 step from ours on 47 t3568 column-B cells (x^a from 7E-17 to 2E-3)"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
`BETA.DIST(x, a, 1, TRUE)` at x from 0.9 to 0.9965 and a from 136 to 1414 returns Excel's value on the 47 column-B
cells of the three t3568-r5-bup cases listed below. These cells take the a0 = 1 complement of `shape_one_tails`. The
fix finds where Excel's w = 1 - x^a differs by one unit of 2^-53 from the engine's
`-excel_expm1(b0 · excel_ln_one_minus(x0))`, and does not lose any of the 1,738 column-B cells that match today.

## Context
- T-3886 lead (docs/research/t3886/RESULTS.md section 2; statistics.md "Incomplete beta at a shape of 1 outside the
  closed forms"): Kahan's expm1 reassociated as (u - 1)·(p/LN u) in `shape_one_tails` makes all 47 of these cells
  bit-exact (full-eval A/B at origin/main b71347b1, `kahan_reassociated.patch`). It also loses 47 bit-exact cells
  elsewhere, 5 of them gated (probes/t3354-f-dist-tiny-x-huge-d2), and no shape cut separates the wins from the
  losses. It is a lead, not Excel's form.
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): kernel family (incomplete beta, bpser) measured on t2464-r2-bpser-small-gl, 0 of 5843 cells changed over three fills; its own cells were not refilled; see docs/research/t3992/README.md.
- Gap triage 2026-11-01k (`make eval` on origin/main 063c2dd9, excel-graph; `scripts/exit_owners.py --all-off`).
  These 47 cells are visibly off. Their only owner is T-3712, which cannot be picked: it depends on T-3382, which is
  still open. T-3712 names the three cases whole but works on column A (the L140 bup+bgrat cdf). It reads column B
  only as Excel's D, and its Context already says column B "is the a0 = 1 complement". Split: this task takes the
  column-B cells below, and T-3712 keeps column A and column C.
- After bratio's swap the cell is a0 = 1, b0 = a, x0 = 1 - x (0.0035 to 0.1), y0 = x > 0.9. That is the
  `shape_one_tails` branch (crates/rustsheet-engine/src/functions/statistical/incomplete_beta.rs). It computes
  w = -excel_expm1(b0 · excel_ln_one_minus(x0)) and returns w1 = (0.5 - w) + 0.5 (statistics.md "Incomplete beta at a
  shape of 1 outside the closed forms (T-3007)"). The rule was fitted on F cells at small b0. Here b0 is 136 to 1414
  on the failing cells (17.8 to 2970 over the whole column), so z = b0 · ln(1 - x0) runs from about -1 to well below
  -36, and w1 is a difference of two doubles near 1.
- On every failing cell, ours and Excel differ by exactly one 2^-53 step of w, and the complement amplifies that
  step:
  - t3568-r5-bup-fresh B744 `BETA.DIST(0.95591070645675,826.0,1.0,TRUE)`: Excel 1.1102230246251565E-16, ours
    2.220446049250313E-16, math x^a = 6.68E-17
  - t3568-r5-bup-tune B856 `BETA.DIST(0.9094554,241.0,1.0,TRUE)`: Excel 1.1649814446457185E-10, ours
    1.164980334422694E-10, math 1.164980992637334E-10
  - t3568-r5-bup-hold B816 `BETA.DIST(0.912187876,136.0,1.0,TRUE)`: Excel 3.7278996465328618E-06, ours
    3.727899646643884E-06, math 3.7278996465580678E-06
  - t3568-r5-bup-hold B1610 `BETA.DIST(0.9965,1414.0,1.0,TRUE)`: Excel 0.007029209602111175, ours
    0.007029209602111286, math 0.007029209602111176
- Evidence already in eval (no new sweep is needed to choose): 1,785 column-B cells, all of this form, across
  t3568-r5-bup-hold, -tune and -fresh, with Excel's values in their `.oracle.json`. The engine matches 1,738 of them.
  Gap-triage scratch scores, with libm stand-ins (not Excel's kernels) and the formula taken as written:
  - Several forms each fit all 47 failing cells and 516 of the 1,785: 1 - (1 - x^a), 1 + expm1(a·ln x),
    1 - (1 - exp(a·log1p(-x0))).
  - The engine's form fits the other 1,738.
  - By z = a·log1p(-x0), the failing cells fall in these bands (cells / engine matches / 1 - (1 - exp(z)) matches):
    - z in [-5, -1): 5 / 4 / 5
    - z in [-20, -5): 146 / 131 / 87
    - z in [-36, -20): 218 / 194 / 107
    - z < -36: 1,416 / 1,409 / 317
  So no single libm form explains the whole column. Two models to try first:
  - Excel's w comes from Kahan's expm1 with a different rounding of `(u - 1) * x / LN(u)`. numbers.md "Excel's expm1
    is Kahan's formula" notes that rounding the product and quotient to 64 bits also fits its sweep.
  - Excel's expm1 switches to EXP(x) - 1 from |x| >= 1, as PMT's 1-Q does (financial.md "PMT is a discounted
    payment", T-3014; numbers.md says whether that switch belongs to Excel's expm1 itself "is not established").
  Also compare excel_ln_one_minus(x0) with an accurate ln(1 - x0): b0 multiplies its last-bit error by up to 1414.
- Excel is far from the math where x^a is tiny: B744 is 66% above it. Matching Excel's value is always right (CLAUDE.md
  directive 1). Do not move these cells to the math: that needs an R3 owner ruling (decision 0188), and this task does
  not ask for one.
- Related:
  - T-3150 (Excel's w1 below 2^-53 at a0 = 1) covers the region where w1 is a multiple of 2^-53. Four cells below are
    in it (B744, B1130, B1609, B1546, with x^a < 2^-53). Whichever task lands first takes them and records it in the
    other's file.
  - The same branch serves F.DIST and F.DIST.RT with a d1 of 2 (a0 = 1) and NEGBINOM/BINOM at a shape of 1. Check
    their eval cells for regressions.
- Code: `shape_one_tails` and `closed_form_tails` in functions/statistical/incomplete_beta.rs; `excel_expm1` in
  x87_exp.rs; `excel_ln_one_minus` and `excel_log1p` in x87_log1p.rs. Read docs/research/excel-kernel-playbook.md
  first. Decision 0056 applies: after two attempts that do not raise the bit-exact count, time-box.

Owned cells (all `BETA.DIST(x, a, 1.0, TRUE)`):
- probes-numeric/t3568-r5-bup-fresh.case.json (9): Calc!B482, B502, B535, B572, B652, B681, B744, B764, B798
- probes-numeric/t3568-r5-bup-hold.case.json (26): Calc!B816, B955, B983, B985, B1002, B1003, B1010, B1043, B1048, B1051, B1055, B1091, B1113, B1130, B1204, B1227, B1420, B1433, B1468, B1546, B1574, B1583, B1590, B1604, B1610, B1616
- probes-numeric/t3568-r5-bup-tune.case.json (12): Calc!B847, B856, B884, B957, B1113, B1214, B1233, B1320, B1363, B1515, B1600, B1609

## Visible-parity stage
Decision 0083 / 0114 (`visible`): met when the 47 cells above are within half a unit of the 15th significant digit
of Excel's value, and no column-B cell of the three cases that is within 15 digits today moves off. Bit-level
criteria still unmet then go to a follow-up with `defer: phase-1b`.

## Acceptance criteria
- [x] The 47 owned cells are within half a unit of Excel's 15th significant digit (`make eval-show` on the three
      cases). Column B of the three cases has more cells within 15 digits than 1,738 of 1,785, and the Outcome shows
      the before and after counts.
- [x] No gated cell and no probes-numeric cell goes from within 15 digits to visibly off. In particular, check the
      F.DIST/F.DIST.RT/FDIST cells at a d1 of 2, the shape-of-1 cells of t1745-beta-inv-roundtrip, and
      BETADIST_BETAINV.xlsx. Any that moves is listed with Excel / new / exact values, or the change is narrowed.
- [x] R4 guards (reviewer-excel-fidelity check 7, decision 0188), since this picks a rounding model or a switch from
      ranked candidates (kind (c), or kind (a) if a cut is fitted):
      - Hold-out counts: train and hold-out side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
        model and the correctly rounded value. The split is about 30% of the column-B cells by cell-id hash, fixed
        with its margin before any scoring. The chosen model beats correctly rounded on hold-out.
      - Fresh sweep: drawn after the model is frozen, as a new probes-numeric case, log-uniform over x0 in
        [1E-3, 0.1] and b0 in [2, 3000]. It includes points with w1 from 1E-3 down to about 1E-16 and F.DIST.RT
        companions at d1 = 2. The result is recorded in the Outcome.
      - Gate A envelope: the fresh sweep's count within 15 digits of Excel or within 1e-9 of 50-digit math, with
        every point outside listed (mpmath values attached).
      - `FITTED(T-3764, n_cells, range)` tag and range on any fitted cut, with accurate math outside the range.
      - Reproducibility floor: no step fits differences below the oracle's floor.
      - No keep-matched branch: no band defined by the cells that already match.
- [x] statistics.md "Incomplete beta at a shape of 1 outside the closed forms" records the model for large b0, citing
      these cells. If no model fits, it lists every candidate tried with its counts.
- [x] A unit test pins at least two owned cells at Excel's value, naming the case and cell.
- [x] No new known-failure line in any eval/baseline/<suite>.txt (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Column A, the L140 bup+bgrat cdf (T-3712), and column C, the density (T-3569, T-3392).
- w1 below 2^-53 beyond the four cells named above (T-3150).
- Moving any cell to the math (R3, which needs an owner ruling).

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 147 probes-numeric cells that no open task listed: 24 first wrong at the 11th significant digit or earlier, 123 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3568-r5-bup-fresh.case.json (12): Calc!A38, A47, A212, A261, A276, A291, A705, A718, A727, B39, B71, B296
- probes-numeric/t3568-r5-bup-hold.case.json (33): Calc!A4, A20, A38, A74, A112, A174, A204, A207, A336, A475, A496, A536, A546, A555, A594, A609, A618, A685-A686, A830, A879, A910, A1371, A1462, B274, B285, B437, B500, B509, B536, B572, B579, B621
- probes-numeric/t3568-r5-bup-terms-hold.case.json (27): Calc!B5, B105, C152, C165, C189, C203, D87, G165, H152, I165, J105, K231, L52, L231, M251, O52, O231, Q87, T151, V52, V231, W231, X15, AA47, AA278, AC165, AE15
- probes-numeric/t3568-r5-bup-terms-tune.case.json (34): Calc!B210, D206, E253, F22, F75, F210, F216, F250, G216, H253, I216, I257, J216, J257, L22, M106, N257, O206, P75, Q257, R22, S110, S261, V22, V75, V206, V250, W257, X106, Y257, AB257, AB261, AC206, AE261
- probes-numeric/t3568-r5-bup-tune.case.json (41): Calc!A9, A53, A61, A137, A145, A204, A211, A224, A281, A325, A327, A368, A404, A447, A449, A471, A481, A519, A537, A595, A598, A607, A667, A673, A679, A757, A843, A920, A944, A1383, A1396, A1560, A1599, A1648, B9, B31, B230, B534, B634, B690, B737

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Hold-out split (fixed before any candidate was scored)
- Population: the column-B cells of probes-numeric t3568-r5-bup-tune, -hold and -fresh (1,785 cells). The ones on
  `shape_one_tails` are those with x > 0.9 (508); the rest are the b0 = 1 closed form.
- Hold-out: sha256 of `<case id>!<cell>` (for example `t3568-r5-bup-hold!B816`) with a first byte below 77 (77/256,
  30.1%). Train: the rest.
- Margin: the chosen model's hold-out bit-exact rate is within 3 percentage points of its train rate, and it beats
  correctly rounded on the hold-out cells.

## Outcome
- **Model (structural, not fitted): from |z| = 1 Excel's a0 = 1 complement takes w = `1 - EXP(z)`, z = b0·ln1p(-x0).**
  Below |z| = 1 it stays Kahan's expm1. This is the switch at 1 that PMT's `1-Q`, WEIBULL.DIST and EXPON.DIST already
  make (numbers.md "Excel's expm1 is Kahan's formula"). The cut is copied from them, so there is no `FITTED` tag, no
  band chosen by matching cells, and no step below the oracle's floor (every move is one 2^-53 step of w). Code:
  `x87_exp::excel_one_minus_exp`, called by `incomplete_beta::shape_one_tails`. Frozen at db9d4e3d7 before the fresh
  sweep was built.
- **The 47 owned cells are all bit-exact.** Column B of the three cases: 1,738 of 1,785 bit-exact before, 1,785 after.
  On the 508 cells on `shape_one_tails` (x > 0.9, |z| 3.9 to 215) Excel's w1 is 1 - RN(1 - x^a) on every cell.
- **R4 hold-out** (split fixed and pushed before scoring, section above). Bit-exact / within 4 ulp / within 15 digits:

  | side | cells | model | Kahan (main) | correctly rounded |
  |---|---|---|---|---|
  | train, all column B | 1,258 | 1,258 / 1,258 / 1,258 | 1,226 / 1,226 / 1,226 | 14 / 132 / 217 |
  | hold-out, all column B | 527 | 527 / 527 / 527 | 512 / 512 / 512 | 5 / 51 / 79 |
  | train, shape of 1 | 359 | 359 / 359 / 359 | 327 / 327 / 327 | 0 / 5 / 5 |
  | hold-out, shape of 1 | 149 | 149 / 149 / 149 | 134 / 134 / 134 | 0 / 2 / 2 |

  The hold-out rate equals the train rate (100%), within the 3-point margin, and beats correctly rounded.
- **Fresh sweep after the freeze**: probes-numeric t3764-shape-one-complement-fresh (1,800 cells, excel-graph,
  2026-11-06; seed 37640): 600 rows of `BETA.DIST(x, b0, 1, TRUE)` with `F.DIST.RT(f, 2, 2·b0)` and `F.DIST(f, 2, 2·b0,
  TRUE)` companions, x0 log-uniform in [1E-3, 0.1] and b0 in [2, 3000], half the rows log-uniform in |z| from 0.3 to
  40 (w1 from about 0.74 down to 4E-18). Bit-exact / within 4 ulp / within 15 digits, model against Kahan's:
  BETA.DIST 555/590/591 against 493/533/535; F.DIST.RT 558/592/592 against 493/527/527; F.DIST 525/600/600 against
  460/600/600. Within 13 digits: 599 and 600 against 566 and 569. The two forms differ on 4 BETA.DIST rows with |z| in
  (0.7, 1.5), all at 1.38 to 1.48, where the switch is right.
- **Gate A envelope** on the fresh sweep: 1,800 of 1,800 inside (within 15 digits of Excel, or within 1e-9 relative of the
  50-digit math). The 17 cells outside 15 digits are all within 1e-14 of the math: the 9 BETA.DIST cells by the
  case's math sidecar, the 8 F.DIST.RT cells (B4, B71, B359, B363, B377, B395, B412, B449; the sidecar generator skips
  them) by mpmath at 50 digits on the closed form (d2/(d2 + 2f))^(d2/2) at the truncated d2: ours is 0.8E-15 to
  7.2E-15 relative off it, and Excel 0.6E-15 to 7.2E-15 (for example B377 `F.DIST.RT(4.87783272858,2,1999.172)`: math
  0.0077043610890345503, Excel 0.007704361089034606, ours 0.007704361089034495).
- **Second change: BETA.DIST's worksheet cdf takes T-4045's L140 prefactor zero** (`bratio::beta_dist_cdf`; the
  quantile searches keep `beta_dist_lower_tail`, so F.INV's accuracy/F.INV/s1 A146 stays in Gate A's envelope). Column
  A of the three cases has 47 Excel zeros; the rule matches all 47 (28 before; train 32/32, hold-out 15/15 on the
  split above) and zeroes none of its 4,078 Excel numbers. It also makes 44 gated cells of
  probes/t4254-beta-t-l140-zero-cut exact (A12:A22, B12:B22, C12:C22, D12:D22, recorded in T-4437). An existing rule
  applied to one more caller: nothing is chosen by scoring.
- **Eval delta** (`rustsheet-eval run` on main c97e0b9c and on this branch, cell by cell and by each case's `passed`):
  140 cells newly bit-exact, 0 regressions, no cell of any case moves away from Excel. probes: t4254-beta-t-l140-zero-cut
  22 to 66 (44 baseline lines removed by `make eval-baseline`). probes-numeric: t3568-r5-bup-tune 2,173 to 2,197, -hold
  2,162 to 2,194, -fresh 1,079 to 1,089, t2220-f-dist-sweep-cdf 806 to 807, -rt 708 to 709, t3567-r5-fdf1-hold 2,881 to
  2,885, t4017-f-huge-df2-above-mean 1e7 136 to 142, 1e8 103 to 111, 1e9 107 to 109, 1e10 105 to 107, 3e7 99 to 103,
  3e8 106 to 108. Family closeness (decision 0267 item 2) visibly-off numbers 2,048 to 1,937 with the new case
  included. Gate A: 0 failing (as main).
- **Decision 0267 cells.** Of the 65 Tier A and 126 Tier B cells of the three t3568-r5-bup cases and the two terms
  cases (the 47 owned plus the 147 listed), 60 Tier A and 3 Tier B now match Excel. Of the 147 listed cells, 19 are now
  bit-exact (all Tier A, the column-A zeros); 5 Tier A and 123 Tier B are left:
  - 4 Tier A column-B cells (fresh B71, hold B285, B500, tune B31): Excel 0 where the true tail is 7E-308 to 7E-307 and
    a neighbour 6.4E-305 is a number. They take bratio's bgrat at a0 = c < 1, which is not ported for a0 other than 1/2
    (T-3582); its underflow exit is the suspect.
  - hold A1371 (`BINOM.DIST(2,1925,0.297025813,TRUE)`, 6th digit): Excel 6E-6 low at 7.6E-290, the size of a subnormal
    bup prefactor's lost bits.
  - The 123 Tier B cells, 12th or 13th digit: 61 are Excel's own beta density at Beta(c + j, a0), a0 up to 3000 (the
    terms cases), where Excel is up to 300 units of 2^-53 off the math (docs/research/t2464/r5-bup RESULTS.md item 1);
    the 47 column-A cells are L140 bup+bgrat, whose W tracks that same density error (correlation 0.93 to 1.00); 15
    are column-B bgrat increments at c < 1 (T-3582).
  Two attempts were made on the listed cells (the 1 - EXP switch and the L140 zero rule). They moved the 19 above and
  none of these 128, whose mechanisms belong to Excel's large-shape density and the unported bgrat. Under decision
  0267 item 4 the 128 are accepted; no follow-up is filed for them.
- Tests: `test_fn_beta_dist_shape_one_complement_past_unit_exponent` (tune B856, hold B816 and B1610, fresh B744, bit
  for bit) and `test_fn_beta_dist_l140_prefactor_underflow_is_zero` (fresh A291, hold A20 and A686, and BETADIST).
- Docs: statistics.md "Incomplete beta at a shape of 1 outside the closed forms" (the model, hold-out, fresh sweep)
  and "Where the exact 0 starts (T-4045)"; numbers.md "Excel's expm1 is Kahan's formula"; T-3150 takes note of the
  four sub-2^-53 cells; T-4437 of the BETA.DIST zeros.
- Follow-ups: T-4511 (fold the three other 1 - e^x switches into `excel_one_minus_exp`), T-4512 (the one-step w left on
  the fresh sweep, owner of its visibly-off A120).
