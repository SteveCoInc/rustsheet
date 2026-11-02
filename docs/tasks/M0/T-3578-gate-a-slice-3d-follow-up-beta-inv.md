---
id: T-3578
title: "Gate A slice 3d follow-up: BETA.INV s1 table (excel-graph refuses the generated workbook) and the GAMMA.INV/CHISQ huge-shape batch with its R3 points"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
The rest of T-3529: a filled BETA.INV s1 Gate A table, and a huge-shape batch for GAMMA.INV, CHISQ.INV and
CHISQ.INV.RT with the decision 0188 R3 points of "GAMMA.INV at a huge shape, closed-form branch" marked.

## Context
- T-3529 landed the definitions of all five functions (`tools/src/rustsheet_tools/accuracy/defs/gamma.py`,
  `beta.py`) and filled tables for GAMMA.INV, CHISQ.INV, CHISQ.INV.RT and BETA.DIST.
- BETA.INV: `rustsheet-accuracy generate BETA.INV` writes 194 points, but `rustsheet-oracle fill -b excel-graph`
  fails at the workbook's createSession three runs in a row (retries exhausted), where the other four tables filled
  in the same runs. Excel has refused workbooks before (statistics.md, FileCorruptTryRepair in T-2435 and T-2982).
  Find the offending formula(s), probably by bisecting the batch, and record the refusal as a quirk.
- Huge shapes: the s1 batches stop at a shape of 1E7 (df 2E7), because the generator's series and continued fraction
  for the incomplete gamma need about 20·sqrt(a) terms. mpmath's `gammainc` gives NoConvergence from a shape of about
  1E5. The R3 region starts at shape 1E9 (closed form from about 2E10). It needs a uniform asymptotic expansion
  (Temme, DiDonato and Morris 1986) in the generator at 50 digits, cross-checked against the series where both run.
  The points then need marks in `math-over-excel.tsv` (protected: a decision record citing 0188 R3's huge-shape
  case), and the engine's points are known failures of T-3364 until it lands.
- The BETA.INV deep-tail R3 points ("Tiny-p quantiles") need the BETA.INV table. CHISQ.INV.RT's tiny-p points in
  T-3529's s1 batch did not show Excel more than G = 1e-4 off the math (see T-3529's Outcome).

## Acceptance criteria
- [x] BETA.INV s1 filled and committed (or a decision on the refused formulas, recorded in docs/excel-semantics).
- [ ] A huge-shape batch for the three gamma quantiles, filled, with the R3 points marked and cited. (Split to T-3863.)
- [x] The beta and gamma `quantile` definitions give no value (NoValueError) where the root is below the smallest
      double, as their docstrings and statistics.md "BETAINV is #NUM! where its quantile underflows (T-2435)" say. Today
      `solve_log_bracket(..., -1500, ...)` returns roots down to about 1E-652. For example, probes-numeric
      t1543-inverse-cdf-tiny-p A91 `BETA.INV(1E-300,0.5,100)` has math 7.87E-603, and CHISQ.INV(1E-300,1) has about
      1.6E-600 where Excel is 0. This was a T-3529 review minor, and it matters once BETA.INV s1 is generated.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes (their owner tasks).

## Outcome
Done in part (2026-11-02, opus run): BETA.INV's table and the underflow bound landed; the huge-shape gamma batch is split
to **T-3863**.
- **BETA.INV s1** (199 points, 12 domain errors), filled from excel-graph on the first attempt. The createSession
  failures T-3529 saw did not recur, so no refused formula was found to record. The generator now drops points whose
  root is below the smallest double (next item), which removed the deep-tail points with sub-double math from the batch.
  27 points are outside the envelope, all known failures with an owner (statistics.md "BETA.INV's Gate A table"):
  - T-3351, 10: Excel 1 - 2^-53 where the quantile rounds to 1, and 2^-53 by the flip at p above 1/2.
  - T-2808, 5: Excel 0.5 at a shape of DBL_MIN.
  - **T-3862** (new), 12: Excel `#NUM!` at a huge alpha and a tiny p.
  `rustsheet-eval accuracy`: 10004 points in 70 tables, 0 failing, 0 table defects, 86 known failures (was 59).
- **Underflow bound, beta only:** `beta.quantile`'s log bracket starts at `LOG_SMALLEST_DOUBLE` (ln 2^-1074, model.py),
  so BETA.INV(1E-300,0.5,100) has no value (Excel `#NUM!`, statistics.md "BETAINV is #NUM! where its quantile
  underflows"). The gamma quantile keeps its bracket on purpose: Excel is 0 there (CHISQ.INV(1E-300,1), and
  accuracy/GAMMA.INV/s1 A82, A85, A145, A155, A172 with math 1E-349 to 7.6E-633). The envelope's 1E-300 floor already
  accepts those points, and a definition change that unvalues committed points would change an immutable batch
  (SPEC-0029, decision 0199). `rustsheet-accuracy verify` on GAMMA.INV, CHISQ.INV.RT and BETA.DIST: no difference but
  this container's 10 s timeouts (GAMMA.INV A140, CHISQ.INV A88, A90, A98; they time out on main too).
- The probes-numeric math sidecars were not regenerated: BETAINV/BETA.INV cells whose math is below the smallest double
  (t1543-inverse-cdf-tiny-p A91) keep their committed values until the next `rustsheet-accuracy math --suite
  probes-numeric` run.
- Tests: tools/tests/test_accuracy_gamma_beta.py (the beta bound, the gamma value kept).
- Eval delta: none (Gate A only; no engine change).
