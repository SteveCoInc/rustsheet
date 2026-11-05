---
id: T-3783
title: "Gate A slice 4m: reference tables for NORM.DIST, NORM.S.DIST, NORM.INV, NORM.S.INV, PHI and GAUSS"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
NORM.DIST, NORM.S.DIST, NORM.INV, NORM.S.INV, PHI and GAUSS each get a committed, oracle-filled `s1` Gate A table
with a cited mpmath definition, and `make check` runs them. Every failing point has an owner and every R3 candidate
is listed.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds") needs a table or an `exempt.tsv` line for every in-scope
  numeric function. On main (fafb6c6f) `rustsheet-eval accuracy` still counts these six M9 functions as missing
  tables. No open task and no pending branch names them for a table: decision 0228 classified only M5, M6 and M7,
  and T-3358's distribution slices stopped at Student t, F, the discrete distributions and T-3529's gamma and beta.
- These are the normal kernels that the later slices build on. LOGNORM.* is T-3785, and the ERF family is T-3784
  (statistics.md, "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)":
  `NORM.S.DIST(x,TRUE)` = `ERFC.PRECISE(-x*SQRT(0.5))/2`, `GAUSS(x)` = `NORM.S.DIST(x,TRUE)-0.5` except for tiny x).
- The math: mpmath `ncdf`/`npdf` and the inverse by a high-precision root of the cdf at the double of each written
  literal. NORM.DIST and NORM.INV draw mean and standard deviation too, and the cumulative flag is a drawn argument.
- Semantics the domain and sampler must read (statistics.md):
  - "The normal CDF is not always correctly rounded" and "Normal CDF and ERFC sweep (T-1040)": the lower tail is up to
    about 1.15E-13 relative from Phi, and 0 from z = -37.625 down. That is a relative error far under decision 0188
    R3's 1e-4, so no R3 point is expected. Any point outside the envelope is a known failure with an owner.
  - "Normal density oracle sweep (T-1041)": PHI equals `NORM.S.DIST(z,FALSE)` bit for bit.
  - "Normal densities are #NUM! once the square overflows (T-2435)": the density is `#NUM!` from |z| = 2^512.
    This is part of the error domain, and T-2260 owns PHI's cut, which is still unprobed.
  - "NORM.S.INV is AS 241 PPND16 in the tails; the central region is unfitted (T-1746)" and numbers.md
    "NORM.S.INV / NORMSINV / NORM.INV(p,0,1): oracle sweep (T-1006)": p reaches about 5E-309, and the central region
    is up to 31 ulp from the correctly rounded inverse.
  - "GAUSS of a tiny argument is x times phi(0) (T-2255)": GAUSS keeps its leading term for tiny z, so its table
    stratum must reach |z| down to 1E-300.
- Likely owners for failing points (check that each is still open when filing): T-0925 (inverse normal), T-0968
  (cdf residue, blocked), T-0975 (pdf last digits). File a new task for a point none of them covers.
- Tools: as in T-3549 and T-3586: `rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`. The definitions go in
  a new `defs/normal.py`, registered in `defs/__init__.py`.
- Why opus: the definitions set the sampler's strata against evidenced Excel cuts (the zero tail, the overflow cut,
  the tiny-z GAUSS term), and each failing point must be told apart as a known failure or an R3 candidate.

## Acceptance criteria
- [x] `tools/src/rustsheet_tools/accuracy/defs/normal.py` has one definition per function, each citing its sources,
      with unit tests against Microsoft's published examples or the semantics entries' values above.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each of the six functions, generated, filled
      (excel-graph) and committed. `rustsheet-eval accuracy` reports `missing tables` lower by 6, with 0 failing points
      and 0 table defects.
- [x] Every point outside the envelope is a `known-failures.tsv` line with an open owner task (T-3379's rules), and the
      Outcome lists them. Any R3 candidate is listed for an owner ruling and is not marked in this PR.
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- The legacy names NORMDIST, NORMSDIST, NORMINV and NORMSINV, and STANDARDIZE, CONFIDENCE.NORM and Z.TEST: the M9
  statistical classification task (T-3786) rules on them.
- Changing the engine's normal kernels: each failing point gets its owner task.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/normal.py`: NORM.DIST, NORM.S.DIST, NORM.INV, NORM.S.INV, PHI and GAUSS, each
  citing its statistics.md entries. Phi is erfc(-z/sqrt 2)/2 through erf.py's `erfc` (its asymptotic branch past
  |x| = 1E+4, where mpmath's own `erfc` overflows), phi is exp(-z^2/2)/sqrt(2 pi), and the inverses solve the smaller
  tail in log scales. GAUSS is erf(z/sqrt 2)/2 below |z| = 1, because Phi(z) - 1/2 cancels every digit at a tiny z.
- Error domain, from the evidence: the densities are `#NUM!` once z*z overflows (T-2435), which covers NORM.DIST,
  NORM.S.DIST and PHI. A standard_dev of 0 or below is `#NUM!`. p is in (0, 1). A cumulative NORM.DIST point whose
  (x - mean)/sd overflows a double is redrawn: no Excel cell says what the cdf is there.
- Left out of the tables, as the ERF slice left out its cancellation region, so the gate does not wait on an R3 ruling:
  - GAUSS on 1E-15 < |z| < 1E-8, where Excel's Phi(z) - 0.5 in doubles is up to 2.6% off. That is gen-m9/146b46e74f2dac3b
    A46, `GAUSS(4E-15)`, Excel 1.5543122344752192E-15 against the math 1.5957691216057308E-15.
  - NORM.INV where mean + sd*z cancels to below 1E-9 of its terms.
- **R3 candidates, for an owner ruling (not marked):** the GAUSS region above, evidenced by gen-m9/146b46e74f2dac3b A46.
  The engine matches Excel there (statistics.md T-2255). In probes-numeric no NORM/PHI/GAUSS cell is more than 1e-4
  off the math.
- Tables: `eval/accuracy/tables/{NORM.DIST,NORM.S.DIST,NORM.INV,NORM.S.INV,PHI,GAUSS}/s1`, 120 to 200 points each,
  filled from excel-graph. `rustsheet-eval accuracy`: 16,472 points in 104 tables, 0 failing, 0 table defects, no new
  known failures. Missing tables went from 153 to 147. The worst Excel-vs-math gap in the six tables is 4.1E-12
  relative (GAUSS(5.69E-06)); the lower tails are within 2.9E-13.
- Sidecars: the probes-numeric math sidecars are regenerated. 19 normal-family cases gain one, and t1543 and t1916
  gain their NORM.INV and NORM.S.DIST cells. An unrelated "why" text drift in t3347-irr-conventional-sweep was not
  committed.
- Unit tests: tools/tests/test_accuracy_normal.py (A&S and Microsoft examples, the T-2255, T-1041 and T-2435
  values, and the draws avoiding both excluded regions).
- Follow-up: T-4277 (owner ruling on R3 for the GAUSS cancellation region, blocked on the owner).
- Eval delta: none in the gated suites (tools, tables and sidecars only).
