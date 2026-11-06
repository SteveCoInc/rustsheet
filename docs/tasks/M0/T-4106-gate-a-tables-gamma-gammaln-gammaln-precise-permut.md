---
id: T-4106
title: "Gate A tables, GAMMA, GAMMALN, GAMMALN.PRECISE, PERMUT and PERMUTATIONA"
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
GAMMA, GAMMALN, GAMMALN.PRECISE, PERMUT and PERMUTATIONA each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- GAMMA and GAMMALN are Excel's own x87 routines, and Excel's value is a valid target within 15 digits (statistics.md
  "GAMMA is Excel's own x87 routine (T-1941, T-2129)", "GAMMALN is Excel's own x87 routine (T-1941, T-2128)", "GAMMA and
  GAMMALN sweep (T-1042)", "GAMMA is exact at small positive integers but not always correctly rounded").
  GAMMALN.PRECISE shares GAMMALN's kernel.
- PERMUT and PERMUTATIONA have integer results with a floating-point stage, so they need a table, as FACT and COMBIN did
  in decision 0228 (statistics.md "Overflowing arguments, PERMUT's limit and argument error order (T-1290)"). Reuse
  `defs/combinatorics.py`.

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [x] `make check` green.
- [ ] Reviewers approve.

## Out of scope
- The other groups of decision 0272.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/gamma_function.py` (new): GAMMA (Γ(x)), GAMMALN and GAMMALN.PRECISE
  (ln Γ(x); one shared definition registered under both names, as Excel's one routine).
  - Domain errors (Excel's `#NUM!`): GAMMA at 0 and the negative integers, and where Γ(x) overflows a double (the math's
    last finite double is Excel's, 171.6243769563027); GAMMALN at x <= 0. GAMMA's reflection underflow to 0 below about
    -171 (statistics.md T-2129) is no domain rule: the math there is below the envelope's 1E-300 floor.
  - Its own draw: typical (0.05, 10) and (-10, -0.05), tails toward the overflow and near the poles 0 to -20, edges
    (integers, half-integers, poles, the overflow, the reflection underflow, the kernel cuts), extreme (10, 171.62) and
    (-1000, -10), log-uniform. GAMMALN uses the stock sampler on (0, inf).
- `defs/combinatorics.py`: PERMUT (n!/(n-k)!, both floored) and PERMUTATIONA (n^k, n floored, k truncated), reusing
  COMBIN's helpers; the overflow check moved to `_common.finite_double` (shared by FACT, COMBIN, GAMMA, PERMUT).
  Domain errors: a negative argument, PERMUT's k > n, a number of 2^31-1 or more, a result past the double range, and
  (new, found here) a PERMUTATIONA number_chosen of 2^31-1 or more. A negative or fractional-negative number and
  PERMUTATIONA's 2^31-1 number cut were not drawn and stay unprobed (said in the module comment).
- Tests in tools/tests/test_accuracy_gamma_permut.py (A&S values, the semantics entries' Excel values, domains, draws).
- Tables `eval/accuracy/tables/{GAMMA,GAMMALN,GAMMALN.PRECISE,PERMUT,PERMUTATIONA}/s1`, 120 points each, filled from
  excel-graph with 0 failures. The PERMUTATIONA math sidecar was regenerated before commit once the oracle showed the
  number_chosen rule (the case file is byte-identical, so its oracle sidecar stands).
- `rustsheet-eval accuracy`: 600 new points, 597 inside the envelope (595 on both sides, 2 PERMUTATIONA points through
  the math only), 3 known failures added (PERMUTATIONA/s1 A71, A76, A79, owner T-4501, filed here: Excel `#NUM!` at a
  number_chosen of 2^31-1 or more, the engine 0 or 1). 0 failing. `missing tables` 121 to 116.
- Engine bit-exact with Excel on 537 of 600: GAMMA 107/120, GAMMALN 111/120, GAMMALN.PRECISE 114/120 (the misses are
  the documented few-ulp kernel gaps, T-2122), PERMUT 120/120, PERMUTATIONA 85/120 (see below).
- New quirk, statistics.md "PERMUTATIONA is binary powering in doubles, and #NUM! at a huge number_chosen (T-4106)":
  Excel's n^k is right-to-left binary powering with every product rounded to a double (104 of 105 numeric cells;
  `powf`, the engine, 73), except `PERMUTATIONA(10,308)`, exactly 1E+308; and a number_chosen of 2^31-1 or more is
  `#NUM!` even for 1^k and 0^k. Both are T-4501.
- Excel against the math: the largest relative gap on the numeric points is 7.2E-15 (`GAMMA(166.471380426933)`).
  Points more than 1e-4 off, Excel kept as the target (decision 0300): 8 GAMMA points from -182.5 to -783.2
  (`GAMMA(-182.466518925445)`, `(-200.678514072972)`, `(-291.301666676081)`, `(-293.397855269841)`,
  `(-402.679890472857)`, `(-565.044249054827)`, `(-652.805344740505)`, `(-783.246466404887)`), where Excel's
  reflection underflows to 0 and the math is below 1E-335 (documented, statistics.md T-2129; inside the 1E-300 floor).
- Math sidecars of the probes and probes-numeric cells calling these five functions regenerated
  (`rustsheet-accuracy math --function ...`); `make eval-math-check` current.
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
