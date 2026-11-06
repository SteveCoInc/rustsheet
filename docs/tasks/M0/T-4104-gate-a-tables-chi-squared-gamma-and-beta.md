---
id: T-4104
title: "Gate A tables, chi-squared, gamma and beta aliases: CHIDIST, CHIINV, GAMMADIST, GAMMAINV, BETADIST and BETAINV"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3925]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
CHIDIST, CHIINV, GAMMADIST, GAMMAINV, BETADIST and BETAINV each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
Every failing table point has an owner.

## Context
- Gate A (decision 0188 R1(a), SPEC-0029): decision 0272 (T-3798) classified these functions as needing a table.
- A legacy name gets its own table that reuses the modern function's Definition (the default of T-3798: TINV already has
  its own table beside T.INV.2T). Where the alias's arguments differ (tails, a right tail, a cdf only), the shared
  Definition takes that into account rather than being copied (directive 3).
- An Excel error region is part of the domain or a `domain` rule, never a math value. Where Excel's number is more than
  G = 1e-4 off the math, list the points in the Outcome as R3 candidates for an owner ruling and do not mark them
  (decision 0188 R3).
- Tools: as in T-3925 (`rustsheet-accuracy` table generation, oracle fill, `rustsheet-eval accuracy`).
- Reuse `defs/gamma.py` and `defs/beta.py` (T-3529) and T-3925's GAMMA.DIST and CHISQ.DIST.RT definitions. CHIDIST is
  the upper tail.
- Sources (statistics.md): "Incomplete gamma sweep (T-1045)", "Chi-squared quantile at df 2 and gamma quantile at alpha 1
  are closed forms (T-2860)", "BETAINV is #NUM! where its quantile underflows, but 0.5 at a tiny alpha and a high p
  (T-2435)", "Legacy distribution names sweep (T-1047)", "Legacy distribution names sweep, second family (T-1048)".

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.

## Outcome
- `defs/legacy_gamma_beta.py` registers CHIDIST, GAMMADIST, GAMMAINV and BETAINV as their modern Definitions
  (CHISQ.DIST.RT, GAMMA.DIST, GAMMA.INV, BETA.INV); CHIINV's definition landed meanwhile as `gamma.CHIINV` (T-3337,
  #3563), and its table here verifies against it with 0 differences under the legacy name (`dataclasses.replace`, one
  definition per concept), citing the T-1047 and T-1048 sweeps. BETADIST is BETA.DIST's cdf without the flag (its value
  and domain wrap BETA.DIST's with TRUE inserted; the bounds follow). tests/test_accuracy_legacy_gamma_beta.py checks
  Microsoft's published examples and the paired modern values.
- `gamma.quantile` (shared with GAMMA.INV and the chi-squared quantiles): at a tiny shape (GAMMAINV's s1 drew 4.8E-266)
  Q = 1 - P loses all its digits and its log went complex; such a point now has no math value (NoValueError) and is
  redrawn. `rustsheet-accuracy verify` on GAMMA.INV, CHISQ.INV and CHISQ.INV.RT: 0 differences in every committed batch.
- Six oracle-filled `s1` tables (excel-graph): `rustsheet-eval accuracy` shows 6 fewer missing tables and 0 failing.
  36 points are outside the envelope, each the same behaviour its modern sibling already owns, now listed in
  known-failures.tsv: BETAINV 12 (T-3351, quantile rounds to 1), 5 (T-2808, shape 2.2E-308), 13 (T-3862, huge alpha and
  tiny p); BETADIST 5 (T-3577); CHIINV A93 (T-3364, Excel's Wilson-Hilferty start).
- CHIINV A93: Excel's Wilson-Hilferty closed form, 2.66e-4 off the math, is decision 0188 R3 case 3 (statistics.md
  "GAMMA.INV at a huge shape, closed-form branch", which names CHIINV). Decision 0318 marks it in math-over-excel.tsv on
  decision 0289's terms; it stays a known failure of T-3364, which moves the branch to the math. New evidence: shape 1.29E9.
- statistics.md's T-1047 and T-1048 entries record the six tables as Gate A evidence for the alias rule.
- Math sidecars of the probes and probes-numeric cells that call the six names (`rustsheet-accuracy math --function`).
