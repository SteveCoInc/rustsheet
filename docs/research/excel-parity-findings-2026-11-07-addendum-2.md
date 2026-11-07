# Addendum 2 to the 2026-11-07 Excel-parity findings: NORMINV, ERFC, and audit findings in text/lookup code

Status: research hand-off, **docs only**. These findings follow `excel-parity-findings-2026-11-07.md` and its first addendum.
In a prototype on main `2e192ce48` that combines every fix in the three documents, the full eval reached **Gate B 498/506**
with all gates clean: Gate A exit 0, `check` showing only the three pre-existing probe cases, engine (5757) and xlsx (583)
tests, workspace clippy `-D warnings` and fmt. The probe-suite known failures went from 6315 to 4946. Re-implement through
the work loop and re-measure on current main.

Newly at the Gate B rate here: **NORMINV** and **ERFC**. Still below the rate: NORM.S.INV (5 cells), NORM.INV (4),
LOGINV (1), F.INV (1), IRR (4), NPV, FVSCHEDULE, HYPGEOMDIST. Notes on those are at the end.

## F. NORMINV / NORM.INV centre: keep the inverse-erf product in a register, and scale in NORM.INV's order

**Symptom.** calipers/roundtrip/formula_stress_test/golden.xlsx Statistical!B46, `NORMINV(B45,D41,E41)`: Excel
8.881784197001252E-16 (= 2⁻⁵⁰), ours 6.661338147750939E-16. This is a cancellation cell, `mean + sd·z` ≈ 0.

**Root cause.** For |q| ≤ 0.5 with q = 2(p − ½), the central quantile is `√2 · erfinv(q)`, computed with the small-argument
rational approximation the engine already carries from statrs (Boost-style:
`x·(x+10)·(Y + R(x))` with Y = 0.08913147449493408 and R a degree-7/9 rational). Excel's bits are reproduced when:
- `x+10` is stored, but the product `g = x·(x+10)` stays in an x87 register until the two final summands `g·Y` and
  `g·R(x)` are stored, and then those are summed (extended). Rounding g early, or keeping the shift unrounded, breaks
  the exact controls.
- **NORM.INV / NORMINV** compute `mean + (sd·√2)·u`, i.e. the scale multiplies √2 *before* u.
- **LOGNORM.INV / LOGINV** instead scale the *stored standard-normal quantile*: `EXP(mean + sd·z)` with z already rounded.
  Unifying the two associations breaks ironcalc NORM_DIST.xlsx 'NORM.DIST NORM.INV'!F28:
  `NORM.INV(0.500001994711402,0,1E6)` = 5.0000000001370815; scaling a stored z is one ulp lower.

**Change.**
- Add an `erf::inverse_erf_small(x)` implementing the register discipline above, reusing `poly` and `x87_register::Register`.
  Keep the statrs licence notice next to the coefficients (NOTICE entry).
- In `normal.rs`, use it centrally (|q| ≤ 0.5) in both `normal_quantile` and `standard_normal_quantile`. The PPND16 tail
  path is unchanged; elsewhere use `−√2·erfc_inv(2p)`.
- In `log_normal.rs`, LOGINV becomes `EXP(mean + sd·standard_normal_quantile(p))`.

**Tests.** Bit-exact tests on calipers Statistical!B42/B46 and on the NORM_DIST.xlsx F28 association control.

**Not solved.** This explains NORMINV's cancellation cell but not the remaining central NORM.S.INV / NORM.INV misses
(about 22 ulp worst). See "Still open".

## G. ERFC (and the CONFIDENCE.T tiny centre): finite gamma-ratio normalization

**Symptom.** gen-m9/984cba67bbe6972b Calc!A128, `CONFIDENCE.T(ERFC(3.14159/Data!C$10),1.999,21)`: Excel
1.9667525376639697E-15, ours 1.966752537663964E-15. ERFC itself was correct; the miss is CONFIDENCE.T's T.INV.2T at a
two-tailed probability close to 1, i.e. a tiny centre.

**Root cause.** In the tiny-centre small-beta inverse, Excel forms the beta normalization from *finite gamma values*
(GAMMA(a)·GAMMA(b)/GAMMA(a+b)), *stores the reciprocal before taking its logarithm*, and keeps the shared beta series'
split exponential. It does not use the extended GAMMALN sum.

**Change.**
- Add a finite-gamma ln-beta helper (`bratio/ln_beta.rs`) and parameterize the existing bounded small-beta Newton solver
  in `inverse_beta.rs` with it; do not duplicate the solver.
- Fall back to the established centre path when a gamma overflows or the first series correction is already resolved.
- `bpser.rs` gains the matching split-exponent option.

**Tests.** Independent controls at df 22, df 28 and df 257, plus the ERFC-confidence cell, are exact.

## Still open (evidence collected so far)

- **IRR (4 cells), and NPV through IRR L17.** The strongest current model is that IRR runs RATE's driver: a uniform
  200-step secant for every input, with *no* flows/peak/guess bounding box. A flat secant run is held and returned when
  |f| < 1e-4. When the secant fails, the retry starts from a *discount-space seed* v = 1.1 (d = 1 − 1.1). That retry
  reproduces `{-400,1,1,1,1,1}` → −0.6737366751808405 at step 125 and `{42,1,1,-1}` → −0.7467044766537371 at step 47.
  The remaining DIV-error cells are *path-dependent* (non-monotonic along an annuity series, e.g. t3406 n=60
  A81/A82 vs A83 vs A84), which argues against any threshold. Excel experiments that would decide:
  - append a trailing zero flow (54 → 55 flows);
  - rescale all flows by 4 (t3478 A206);
  - the power-of-two scaling sweep (T-3117).
- **F.INV (1 cell).** Two classes. Cells with p ≈ 1E-300 stop early on a *flushed subnormal residual*; that path is fragile
  and depends on the start offset. Cells with p ≥ ~1E-290 cannot flush and converge to the CDF crossing.
- **HYPGEOMDIST (1 cell, `HYPGEOMDIST(2,4,3,19)`).** Excel's kernel agrees with BINOM.DIST's for factors with k = n
  (successes = N·p) and differs for factors with m − k = N − n. Direct division forms p best (579/1442 exact over cached
  calls), but no tested structural identity is accepted yet.
- **FVSCHEDULE (1 cell, `FVSCHEDULE(5916,{-1E+300,1E+300})`: Excel 0, ours the DIV error).**
  - *Finite products are already right.* All 130 finite replays match: start with the principal, read rates row-major,
    store `1+rate`, and multiply with a rounding after each step.
  - *Overflow is the only gap,* and our handling of it is unsupported: the DIV error on an infinite result comes from an
    inherited fork line (`if result.is_infinite()`) with no Excel evidence.
  - Smaller overflowing reproductions do not open in Excel for the web, so the overflow rule needs a desktop-Excel probe.
- **NORM.S.INV / NORM.INV centre (≤ 22 ulp).** Fingerprint: every central Excel output is √2·u for some double u, and
  outputs mirror exactly for bit-identical |p − ½|. The best rational starter plus Newton/Halley refinement reaches only
  150/348 exact.

## Audit findings in text, lookup and database code (resource safety, directive 5)

From an independent read-only audit, each confirmed with a repro.

1. **High: text allocation limits bypassed, process abort.**
   - *Where:* `text/common.rs` ~350–364 and `text/arraytotext.rs` ~126–146. The unary transforms (UPPER, LOWER, PROPER,
     TRIM, CLEAN, ASC, DBCS/JIS) and ARRAYTOTEXT allocate without `within_text_budget` or incremental work checks.
   - *Repro:* `=ROWS(UPPER(IF(SEQUENCE(4096),REPT("a",32767))))` builds ~128 MiB and returns 4096, whereas `LEFT(...,32767)`
     returns the VALUE error at the cap. With 65536 rows, or `=LEN(ARRAYTOTEXT(IF(SEQUENCE(2048),REPT("a",32767)),1))`,
     the process aborts (SIGABRT) under a 256 MiB limit.
   - *Fix:* route these through the budgeted text driver, and give ARRAYTOTEXT a bounded builder that charges escaping,
     separators and braces before allocating.
2. **High: wildcard matching is quadratic but charged by input length.**
   - *Where:* `criteria.rs` ~740–798, used by XMATCH and XLOOKUP.
   - *Repro:* `=XMATCH("*"&REPT("a",16000)&"b",IF(SEQUENCE(16),REPT("a",32767)),2)`.
   - *Fix:* use a matcher with a stated complexity bound, or charge token transitions and return a budget error rather
     than "no match".
3. **High: database functions keep scanning after the work budget is exhausted.**
   - *Where:* `database.rs` ~139–157, 283–305, 636–680. D-functions are priced `LiftCost::Unit` regardless of database or
     criteria size.
   - *Repro:* a DCOUNT over n records × n criteria rows grows superlinearly.
   - *Fix:* charge records × criteria cells.
4. **Medium.**
   - The database error-cell special case evaluates volatile selectors twice.
   - TIMEVALUE keeps a second, inconsistent date parser.
   - CELL result metadata is keyed to the tested literal syntax.
5. **Low.** The acknowledged lossy UTF-16 paths still corrupt joins and delimiter matching.

The statistical and financial audit findings are in the first document; one of them, IRR's bounding-box dispatch, is
covered above under "Still open".
