---
id: T-3925
title: "Gate A slice 4p: reference tables for GAMMA.DIST, CHISQ.DIST and CHISQ.DIST.RT"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
GAMMA.DIST, CHISQ.DIST and CHISQ.DIST.RT each get a cited mpmath Gate A definition and a committed, oracle-filled `s1`
table, and `make check` runs them. The math sidecars of probes and probes-numeric are regenerated with the new
definitions, so the 720 R1(c)-counted cells of these three functions that have no math value today are read against
the math. Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): none of the three has a table or an `exempt.tsv` line. T-3798 lists them
  among the 53 M9 functions to classify. Filing this slice lets T-3798 count them as named, as its Goal allows
  ("an open task names it in its title or Goal").
  T-3529 is done, and `tools/src/rustsheet_tools/accuracy/defs/gamma.py` already has the regularized incomplete-gamma
  `lower_tail` and `upper_tail` behind GAMMA.INV, CHISQ.INV and CHISQ.INV.RT. Reuse them (directive 3). The new work
  is the density and the cumulative call shapes.
- Gap triage 2026-11-03 (`make eval`, report.json): these R1(c)-counted probes and probes-numeric cells have no math
  value, because the functions have no definition:
  - GAMMA.DIST, 470: t3582-r6-gamma-runs 220, t3311-gamma-inv-lower-tail-cdf 128, fit-audit-t2638-ratio-min 46,
    fit-audit-t3271-cdf-cut 21, t3316-gamma-cdf-huge-shape 15, and others;
  - CHISQ.DIST, 68: t3311-gamma-inv-lower-tail-cdf 43, t2561-chisq-dist-rt-past-edge 19, and others;
  - CHISQ.DIST.RT, 182: t3285-chisq-dist-rt-half-integer-sweep 73, t3269-fresh-sweep-2 46, t3269-fresh-sweep 26,
    t3284-gamma-deep-upper-tail 24, and others.
  Of these, `scripts/exit_owners.py --all-off` lists 220 GAMMA.DIST and 73 CHISQ.DIST.RT cells that only `defer:` tasks
  own (T-3224, T-3461), so `scripts/phase1_exit.py`'s `owners` criterion reads them as unowned.
- Ownership: until this task lands it owns the counted cells of probes-numeric/t3582-r6-gamma-runs.case.json and
  probes-numeric/t3285-chisq-dist-rt-half-integer-sweep.case.json (decision 0245), because what keeps them counted is
  the missing math value. The Outcome names the owner of every cell of the two cases that is still counted after the
  regeneration, filed if none exists.
- Semantics the domain, error regions and sampler must read (statistics.md):
  - "One incomplete-gamma kernel under ERF, ERFC, NORM, CHISQ, GAMMA.DIST and POISSON (T-1778)";
  - "GAMMA.DIST's density is #NUM! at a subnormal x / beta (T-2738)";
  - "GAMMA.DIST's density below shape 3 is a log-space form (T-2753, T-2852)";
  - "GAMMA.DIST at a huge shape: #NUM! where shape/x overflows; an unexplained #NUM! from about 1E+56 (T-2435,
    T-3316)";
  - "Booleans from a reference or a formula are numbers in the F, beta, chi-squared and hypergeometric functions
    (T-2255)";
  - CHISQ.DIST.RT truncates its df to an integer, and 2b < 1 is `#NUM!` (the incomplete-gamma upper-tail entry, "Q(b,
    z)"). CHISQ.DIST's df: read it from the entries, and do not assume CHISQ.DIST.RT's rule.
  An Excel error region is part of the domain or a `domain` rule, never a math value. Where Excel's number is more
  than G = 1e-4 off the math (deep tails, the huge-shape band), list the points in the Outcome as R3 candidates for an
  owner ruling and do not mark them (decision 0188 R3).
- Tools: as in T-3529 and T-3785 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- Why opus: the density's error regions (the subnormal x/beta `#NUM!` and the unexplained huge-shape `#NUM!`) have to
  be read from evidence, and deep-tail points may be R3 candidates.

## Acceptance criteria
- [ ] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/gamma.py` (or a module it imports),
      reusing `lower_tail` and `upper_tail`, each citing its sources, with unit tests against Microsoft's published
      examples or the semantics entries' values.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each of the three, generated, filled
      (excel-graph) and committed. `rustsheet-eval accuracy` reports `missing tables` lower by 3, with 0 table defects.
- [ ] Every point outside the envelope is a `known-failures.tsv` line with an open owner task. R3 candidates are
      listed in the Outcome, not marked.
- [ ] The probes and probes-numeric math sidecars are regenerated and committed, and `make eval-math-check` is current.
      The Outcome quotes report.md's R1(c) line before and after.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The legacy names GAMMADIST and CHIDIST (T-3798's legacy-alias ruling).
- GAMMA, GAMMALN and GAMMALN.PRECISE (T-3798).
- Any engine change.

## Blocked because
Resumed after the review-round limit (step 5c).
- Round 3 (head 54e0c7109): design and security APPROVE; reviewer-excel-fidelity REQUEST_CHANGES. Findings, all
  addressed in the commit that blocks the task (not yet reviewed):
  - [MAJOR] t2982 A117-A120 (Excel 0, math up to about 1, x / beta underflowing to 0) were made valueless without
    being listed as R3 candidates, unlike t2753 A13-A15: now in the Outcome and T-4392.
  - [MINOR] the docstring claimed Excel's 0 for every subnormal x / beta: now evidenced only where it underflows to 0
    (T-2982), the nonzero subnormal below shape 1 unprobed (T-1942).
  - [MINOR] GAMMA.DIST's density at 0 with shape 1 (1 / beta) was inferred from CHISQ.DIST's df 2: now no value.
    No table point or eval cell is there (`verify` and `math --check` unchanged).
- Open MINORs for T-4395 (or the resumer): `NORMAL_MIN` duplicated with continuous.py; gamma_dist's `RT_TRUNC_SOURCE`
  shares a name with gamma.py's but cites T-0971 (rename, e.g. `RT_DF_TRUNC_SOURCE`); the protected
  `quality/duplication.json` baseline (2878, measured 2876) is the steward's.
- Next: an opus run resumes per work-loop step 5c: `make check-fast`, the full panel on the current head, then CI and
  enqueue. CI on 54e0c7109 was running when the window closed (PR #3412).

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/gamma_dist.py`: GAMMA.DIST, CHISQ.DIST and CHISQ.DIST.RT, each citing its
  statistics.md entries. The cdfs are gamma.py's `lower_tail` and `upper_tail` (P and Q at y = x / beta, x / 2 for the
  chi-squared functions), the tail map is its `quantile`; nothing is copied. The density is y^(a-1) e^-y / (Gamma(a)
  beta) in log space, at log10(a) more digits. The condition-number cut (`ill_conditioned`, `CONDITION_LIMIT`) moved
  from continuous.py to `_common.py` so both modules share one definition (duplicated lines 2878 -> 2876; the protected `quality/duplication.json` baseline is left for the steward to ratchet).
- Error domain, from the evidence: x < 0, shape or scale <= 0, df outside [1, 1E10] (CHISQ.DIST.RT truncating it,
  T-0971's `CHISQ.DIST.RT(8,3.34)`; CHISQ.DIST has no truncation evidence, so a non-integer df gets no math value);
  x / beta overflowing (both forms); the density at a subnormal x / beta (T-2738), where shape / x overflows (T-3316
  rule 1), at 0 with df 1, and at the mode past the 2 pi shape overflow (`#DIV/0!`, T-4308). Excel agrees on every
  error point drawn: 26 `#NUM!` cells across the three tables, each a domain error of the math.
- No value (redrawn; the Excel cells stay in the eval suites): the cdf at a subnormal x / beta where the math is above
  1E-300 (Excel reads the ratio as 0); a value below the smallest normal from a shape of 1E+55 (T-3316 rule 2, the
  unexplained `#NUM!`); the density at 0 below shape 1; the chi-squared density at a subnormal x / 2 or an overflowing
  (df / 2) / x (unprobed); the density off the mode past the 2 pi shape overflow (unprobed); points where the one
  rounding of x / beta moves the value by more than 1e-10 relative (none where the quotient is exact, as x / 2 is);
  huge-shape points Temme's expansion does not reach. Values below e^-2000 are recorded as 0 or 1 (the far tails).
- Tables: `eval/accuracy/tables/{GAMMA.DIST,CHISQ.DIST,CHISQ.DIST.RT}/s1`, 200, 120 and 120 points, filled from
  excel-graph. `rustsheet-eval accuracy`: 19,822 points in 116 tables, 0 failing, 0 table defects, no new known
  failures; missing tables 138 -> 135. The worst Excel-vs-math gap in the tables is 5.2E-14 relative
  (`CHISQ.DIST(3.83524993296558E-22,25,FALSE)`); GAMMA.DIST 4.0E-14, CHISQ.DIST.RT 2.3E-14.
- Sidecars: the 42 probes and probes-numeric cases `rustsheet-accuracy math --check` reported stale (every one-call
  cell of the three functions) are regenerated; `make eval-math-check` is current. A full-suite `math --suite probes`
  run stalled for 30 minutes on cells outside this slice, so only the stale cases were rewritten.
- report.md's R1(c) line, before: "probes 887 counted, 326 Phase 1b evidence, 0 R3 passing, 16 M14-owned;
  probes-numeric 2396 counted, 2925 Phase 1b evidence, 66 R3 passing, 0 M14-owned". After: "probes 788 counted, 425
  Phase 1b evidence, 0 R3 passing, 16 M14-owned; probes-numeric 1916 counted, 3405 Phase 1b evidence, 66 R3 passing,
  0 M14-owned".
- Ownership (decision 0245): every cell of probes-numeric/t3582-r6-gamma-runs (220) and
  t3285-chisq-dist-rt-half-integer-sweep (73) that was counted is now Phase 1b evidence; none is still counted, so
  this task hands no cell on. `scripts/exit_owners.py --all-off --function GAMMA.DIST`: 0 unowned.
- **R3 candidates, for an owner ruling (not marked; T-4392):** probes/t2753-gamma-density-ratio-sweep A13, A14, A15
  (`GAMMA.DIST(1E-300|1E-250|1E-200,2,1,FALSE)`), Excel 0 against the math's about x: the T-2753 log form underflows.
  Recorded in the T-2753 statistics.md entry. Also probes/t2982-gamma-inv-huge-shape A117-A120
  (`GAMMA.DIST(1E-300,a,1E100,TRUE)`, x / beta underflowing to 0, Excel 0 against a math value up to about 1), which
  the definition leaves without a value (listed as skipped), added to T-4392. No other cell of the three functions in the regenerated sidecars is a
  number more than 1e-4 off the math; t4308-gamma-density-r4-fresh A152:A154 (`#DIV/0!`) are the domain rule above
  (the engine fix is T-4340).
- Unit tests: tools/tests/test_accuracy_gamma_dist.py (Microsoft's examples, the T-0971, T-1891, T-1778, T-2753 and
  T-1049 values, the error domain either side of the T-2738, T-3316 and T-4308 cuts, and the no-value regions).
- Follow-up: T-4392 (owner ruling on R3 for the tiny-ratio density), blocked on the owner.
- Eval delta: none in the gated suites (tools, tables and sidecars only).
