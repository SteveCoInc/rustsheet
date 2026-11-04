# 0272 — Gate A classifies the M9 distribution and special functions with no table

- **Status:** accepted (2026-11-04, T-3798).
- **Context:**
  - Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds") needs every in-scope numeric catalog function to have a
    reference table or an `exempt.tsv` line. Decisions 0228, 0270 and 0271 classified the other groups.
  - T-3798 listed 53 M9 distribution and special functions. After decision 0271 all 53 still had neither.
  - Open Gate A table tasks already name 19 of them in their title or Goal:
    - T-3783: NORM.DIST, NORM.S.DIST, NORM.INV, NORM.S.INV, PHI, GAUSS;
    - T-3785: LOGNORM.DIST, LOGNORM.INV, EXPON.DIST, WEIBULL.DIST, FISHER, FISHERINV;
    - T-3925: GAMMA.DIST, CHISQ.DIST, CHISQ.DIST.RT;
    - T-3741: BINOM.DIST.RANGE, BINOM.INV, CRITBINOM;
    - T-3592 names F.DIST.RT and CONFIDENCE.T only as owners of cells inside Gate A, not as a table task, so they are
      filed below.
- **Decision:**
  - **No exemptions.** Every one of the 53 has a floating-point kernel or an integer result with a floating-point stage
    (PERMUT, PERMUTATIONA, BINOM.INV, CRITBINOM), so each needs a table. `exempt.tsv` is unchanged.
  - **Legacy names get their own tables,** reusing the modern function's Definition and taking the alias's arguments
    into account (tails, a right tail, a cdf only). This is T-3798's default. No third "alias" class is added to
    SPEC-0029. TINV already has its own table beside T.INV.2T.
  - **The other 34 are filed as seven table tasks:**
    - T-4100: NORMDIST, NORMINV, NORMSDIST, NORMSINV, LOGNORMDIST, LOGINV, STANDARDIZE (after T-3783 and T-3785);
    - T-4101: CONFIDENCE, CONFIDENCE.NORM, EXPONDIST, WEIBULL (after T-3783 and T-3785);
    - T-4102: T.DIST.2T, T.DIST.RT, TDIST, CONFIDENCE.T;
    - T-4103: F.DIST.RT, F.INV.RT, FDIST, FINV;
    - T-4104: CHIDIST, CHIINV, GAMMADIST, GAMMAINV, BETADIST, BETAINV (after T-3925);
    - T-4105: BINOMDIST, NEGBINOMDIST, HYPGEOM.DIST, HYPGEOMDIST, POISSON;
    - T-4106: GAMMA, GAMMALN, GAMMALN.PRECISE, PERMUT, PERMUTATIONA. Excel's GAMMA and GAMMALN are its own x87 routines
      and a valid target within 15 digits (statistics.md "GAMMALN is Excel's own x87 routine").
- **Consequences:**
  - `missing tables` is unchanged by this record (163 after decision 0271). The eleven table tasks above bring this
    group to zero once they land.
  - With decisions 0270, 0271 and this one, only T-3797 (the M9 financial functions) and T-3786 are left as Gate A
    classification tasks.
