# 0261 — T-3411: fitted lock 9 to 10 in financial_util.rs for IRR_ROOT_RESIDUAL

- **Status:** accepted (2026-11-04, T-3411, round 4 of its R4 plan). It applies decision 0188 R4.3 and adds no rule.
- **Context:** T-3411 makes IRR's fallback (`irr_fallback` in
  crates/rustsheet-engine/src/functions/financial/financial_util.rs, Newton on P(v) from v = 0) `#NUM!` when NPV, as
  `irr_npv` evaluates it one Newton step past the fallback's stop, is 1E-4 or more in magnitude
  (docs/excel-semantics/financial.md, "IRR's fallback is `#NUM!` when NPV at its root is 1E-4 or more"). The cut
  `IRR_ROOT_RESIDUAL` sits between evidenced cells, so it is a fit (decision 0188 R4):
  - It carries `FITTED(T-3411, 169 IRR cases, fallback roots of 2..150 flows, |c| 1E-6..1E+13; cut bracketed
    9.155E-5..1.2207E-4)`. Over every IRR cell in eval no cell moves for a cut anywhere in that bracket.
  - R4.1 split (fit lab `holdout.is_holdout`): 41,288 train and 17,904 test cells. The train cells alone bracket the
    cut at 9.155E-5..1.526E-4; the one test cell inside, probes-numeric/t3478-irr-r4-wide A90 (1.2207E-4), falls on
    Excel's side. Bit-exact against main: train +11, test +4, 0 lost on either side.
  - Fresh sweep probes-numeric/t3411-irr-r4-fresh-4 (150 cells) and the Gate A batch accuracy/IRR/r4-T-3411 (300
    points), both drawn and oracle-filled after the freeze (a94b5f7fc): 5 cells gained and none worse, type mismatches
    67 to 62; Gate A 31 points outside the envelope against 32 on main, the same points and kinds, each carried in
    known-failures.tsv by its IRR task (T-3548, T-3407, T-3408).
  - Round 3 checked at the stop iterate and lost three ungated cells (fresh-2 A43, t3478-wide A79 and A292); round 4
    keeps all three.
- **Decision:** quality/fitted.json rises by one for financial/financial_util.rs (9 to 10), for this one new fit,
  written by `scripts/check_fitted.py --update`.
- **Consequences:** one counted tag. The fitted ratchet stays one-way for new fits, each with its own record. No cell
  of eval is lost against main (`make eval-check`: 0 regressions; probes-numeric scored cell by cell).
