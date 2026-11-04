---
id: T-3925
title: "Gate A slice 4p: reference tables for GAMMA.DIST, CHISQ.DIST and CHISQ.DIST.RT"
milestone: M0
lane: harness
status: todo
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
