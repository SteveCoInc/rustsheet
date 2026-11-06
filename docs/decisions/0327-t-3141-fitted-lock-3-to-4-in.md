# 0327 — T-3141: fitted lock 3 to 4 in annuity.rs for PAYMENT_EXPM1_TO's CUMPRINC use

- **Status:** accepted (2026-11-06, T-3141). It applies decision 0188 R4.3 and adds no rule.
- **Context:** In CUMPRINC, where `v^nper` underflows, the principal over a short range is one core
  `C = pv·((1-v^m)·v^(nper-end+1))`. Its `1-v^m` is PMT's `discount_complement`: Kahan's expm1 below m·L = 1, and
  `1 - EXP` from 1 on (docs/excel-semantics/financial.md, "Where `v^nper` underflows, the principal is one core for
  both types"). Reusing PMT's cut `PAYMENT_EXPM1_TO` for CUMPRINC was picked from evidenced cells, so it is a fit
  (decision 0188 R4):
  - **The bracket.** Of the 25 probed principals where the two forms differ, Kahan's expm1 is Excel's through m·L
    0.9783 (probes/fin-cumprinc-underflow-fresh I4). `1 - EXP` is Excel's at 1.3976 (I5, J5). PMT's 1.0 lies inside,
    so the constant gets `FITTED(T-3141, 25 cells, CUMPRINC underflowed m*L 0.36..1.40; cut bracketed
    0.9783..1.3976)`.
  - **The fit set.** It has 304 underflowed principals: fin-cumprinc-near-1e15, -underflow-ends, -underflow-type1 and
    -underflow-fresh.
  - **R4.1, the fit lab's `holdout.is_holdout` split.** The core is 216/216 to the bit on train and 88/88 on test.
    The correctly rounded value gets 65 and 33.
  - **R4.2, the fresh sweep.** The cut froze on 2026-11-04 (T-3141 branch) and has not changed since. Two aimed
    batches of T-4059 were drawn and filled on 2026-11-06, after that freeze, inside the region: accuracy/CUMPRINC/
    e-T-3141 and e-T-4059, 400 points with rates 1E-4 to 2.9 and m·L 0.003 to 4.8. Of them, 108 are at m·L 0.7 or
    more and 81 at 1.0 or more, and the core is to the bit on all 108. Its one miss over the 400 is e-T-4059 A119
    (m·L 0.24, result 3.2E-285; T-4710), which does not involve this cut.
    - e-T-3141 is fit data for another choice of T-3141, the leading-exponent cut it dropped; e-T-4059, filled after
      that refreeze (0a1f01a), is its aimed hold-out.
    - The full-domain sweep accuracy/{CUMPRINC,CUMIPMT}/r4-T-3141 was drawn after the refreeze. No point is lost
      against main and 4 more are to the bit; 7 of its CUMPRINC points are in the region, and all 7 are exact.
  - **R2, Gate A envelope.** r4-T-3141 has 28 CUMIPMT points outside the envelope, the same on main. All are
    tiny-rate cancellations carried by T-4390. Every CUMPRINC point is inside.
- **Decision:** quality/fitted.json rises by one for financial/annuity.rs (3 to 4), for this one new tag, written by
  `scripts/check_fitted.py --update`. The cut is shared with PMT's and keeps its value.
- **Consequences:** One counted tag. A later change of `PAYMENT_EXPM1_TO` must keep both brackets: PMT's
  0.9896..1.0648 and CUMPRINC's 0.9783..1.3976. No cell of eval is lost against main (`make eval-baseline`:
  0 regressions).
