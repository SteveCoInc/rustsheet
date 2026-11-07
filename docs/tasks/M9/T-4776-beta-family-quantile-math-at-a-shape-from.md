---
id: T-4776
title: "Beta-family quantile math at a shape from 1E9: F.INV, F.INV.RT and FINV at a df of 1E10 and BETA.INV up to 1.3E+154 (36 counted cells with no math value)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math` gives a 50-digit value for the incomplete beta and its quantile at a shape above
SERIES_SHAPE_CAP (1E7), up to the 1.3E+154 of probes/t3862. Then the 36 visibly-off F.INV, F.INV.RT, FINV and BETA.INV
cells below get a math value, and decision 0188 R1(c) can sort them: Phase 1b evidence where both sides are within 1e-9
of the math, a `campaign:` cell or an R3 candidate where Excel is not. Today they count only because they have no math
value (decision 0325). The math, not the counted state, should decide each one.

## Context
- Gap triage 2026-11-07 (`make eval` at b9ae5216f, excel-graph). The cells' sidecars list them under `skipped`:
  `not converged` for the F quantiles at a df of 1E8 to 1E10, `no single value` for BETA.INV at a second shape of 5E9
  and 1.3E+154.
- Root cause (from the code): `tools/src/rustsheet_tools/accuracy/defs/beta.py` `_betainc` hands every shape above
  `SERIES_SHAPE_CAP = 1e7` to `mpmath.betainc` alone. Its alternating 2F1 series does not converge there (the module
  comment cites 75 s for `BETA.INV(0.5,2.5,5E9)` under the old clock). The quantile search over it then disagrees
  between dps 60 and 80, or finds no recordable double (`NoValueError`). `fisher.py` (F.INV, F.INV.RT, FINV) and the
  BETA.INV definition both solve over that `_betainc`.
- What the math is likely to show (triage scratch, the gamma limit I_x(a, b) -> P(a, b x) as b -> infinity, at 40
  digits; its own error is O(a/b), about 5E-10 at b = 5E9, so treat these figures as a guide, not as the math):

  | cell | formula | Excel | ours | Excel - limit (rel) | ours - limit (rel) |
  |---|---|---|---|---|---|
  | probes-numeric/t1538-f-inv-huge-df Calc!A23 | `F.INV(0.025,5,1E10)` | 0.16624247708803258 | 0.16624232267930517 | +9.3e-7 | 0 (to 1e-10) |
  | t1538 Calc!A137 | `BETA.INV(0.025,2.5,5E9)` | 8.312123853710714e-11 | 8.312116133274345e-11 | +9.3e-7 | -1.9e-10 |
  | t1538 Calc!A133 | `BETA.INV(0.5,2.5,5E9)` | 4.351460182692426e-10 | 4.3514601894960433e-10 | -1.9e-9 | -3.7e-10 |
  | probes/t3862-beta-inv-sqrt-dbl-max S!E1 | `BETA.INV(0.1,0.5,1.3E+154)` | 6.041524333826826e-157 | 6.073374651319876e-157 | -5.2e-3 | +2.9e-14 |

  So at a df of 1E10 Excel looks about 1E-6 off the math, and ours looks on it. That is the regime of T-2555 (Excel's
  BETA.INV early Newton stop at a huge shape, blocked, time-boxed, its cells accepted under decision 0267). The
  t3862 E column looks about 0.5% off, past decision 0188 R3's G = 1e-4. That is T-4700's regime (BETA.INV at a huge
  shape below sqrt(DBL_MAX)). An R3 mark needs a decision record with an owner ruling, which is not this task's: report
  the measured gaps in the Outcome and leave the ruling to T-4700. The df 1E8 cells (Calc!A31, A34, A54, A55, A119)
  differ from Excel only at the 13th to 14th digit, and are likely Phase 1b once they have a math value.
- Method is the open design question, hence opus. Candidates: Temme's uniform asymptotic expansion for I_x(a, b) at a
  large shape (DiDonato and Morris's basym is its double-precision form); or the gamma limit with its 1/b correction
  series carried to 50 digits; or `_tail_series` with a budget that scales with the shape but stays deterministic (no
  clock, T-3680). Whatever is chosen has to agree at dps 60 and 80, and has to leave every existing Gate A table value
  unchanged (`rustsheet-accuracy verify BETA.DIST`, `BETA.INV`, `F.DIST`, `F.INV`).
- Scope, so this task merges independently of T-4775 (the BINOM.DIST and NEGBINOM.DIST cdfs, `defs/discrete.py` only):
  this task changes `defs/beta.py` (and `fisher.py` only if its search needs it). T-4036 (sonnet) restructures
  `_betainc` without changing a value. Whichever lands second rebases.
- Related, not listed here: probes-numeric/t3443-basym-sweep, -a and -hold have 16 `not converged` composite cells
  (`F.INV(F.DIST(x,d1,d2,TRUE),d1,d2)` at d2 1E5 to 2E5, owned by T-3597). Their shapes are below the cap, so their
  failure may be a different one. Check whether this change gives them values. If it does, say so in the Outcome; if
  not, leave them to T-3597.

Cells this task owns for their math value (fix work stays with T-2555, T-4700, and for t1530 T-4435, T-2642, T-3209,
T-3766):
- probes/t3862-beta-inv-sqrt-dbl-max.case.json (6): S!E1, S!E2, S!E3, S!E4, S!E5, S!E6
- probes-numeric/t1530-f-df-cut.case.json (2): Calc!A3, Calc!A5
- probes-numeric/t1538-f-inv-huge-df.case.json (28): Calc!A4, Calc!A18, Calc!A23, Calc!A26, Calc!A31, Calc!A34, Calc!A37, Calc!A40, Calc!A46, Calc!A47, Calc!A54, Calc!A55, Calc!A60, Calc!A61, Calc!A68, Calc!A89, Calc!A111, Calc!A114, Calc!A119, Calc!A125, Calc!A128, Calc!A131, Calc!A133, Calc!A135, Calc!A137, Calc!A139, Calc!A153, Calc!A155

## Acceptance criteria
- [ ] `_betainc` returns I_x(a, b) at a shape above 1E7 with dps 60 and 80 in agreement, and the BETA.INV and F
      quantile searches over it record a double for every cell above. A unit test pins at least three of them
      (t1538 Calc!A23, A133, t3862 S!E1) against an independent 50-digit reference: the gamma limit with enough
      correction terms, or mpmath quadrature of the beta density.
- [ ] `cd tools && uv run rustsheet-accuracy math --suite probes` and `--suite probes-numeric` regenerate the three
      sidecars, and none of the 36 cells is under `skipped`. `rustsheet-accuracy math --check` passes, and
      `rustsheet-accuracy verify` shows no change for BETA.DIST, BETA.INV, F.DIST, F.DIST.RT, F.INV and F.INV.RT.
- [ ] After `make eval`, the Outcome lists each of the 36 cells as Phase 1b evidence (`r1c_left_out`) or with its
      Excel-minus-math and ours-minus-math, and names the cells where Excel is more than 1e-4 off (R3 candidates for
      T-4700). `eval/baseline/` gains no new line (this task changes no engine value).
- [ ] The budget stays deterministic, and `make check` is green; reviewers approve (excel-fidelity included).

## Out of scope
- Any engine change to BETA.INV or the F quantiles (T-2555, T-4700, T-4435).
- An R3 (`MATH-OVER-EXCEL`) mark or decision record for the t3862 cells (T-4700, owner ruling).
