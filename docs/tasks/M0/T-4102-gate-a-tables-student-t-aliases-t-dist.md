---
id: T-4102
title: "Gate A tables, Student t aliases: T.DIST.2T, T.DIST.RT, TDIST and CONFIDENCE.T"
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
T.DIST.2T, T.DIST.RT, TDIST and CONFIDENCE.T each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse `defs/student_t.py` (T-3358). TDIST takes `tails`. CONFIDENCE.T is T.INV.2T(alpha, n-1)·sd/sqrt(n).
- Sources (statistics.md): "T.DIST.RT and T.DIST.2T are not 1 - T.DIST", "T.DIST.RT follows the rounded y beyond t = 1
  (T-0969)", "T.DIST.RT is BETA.DIST's incomplete beta, switched at the mean (T-0969)", "T.DIST.RT's noise beyond t = 1
  grows with a ln y (T-0969)". T-3592 owns 1 CONFIDENCE.T cell inside Gate A.

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/student_t.py` now holds T.DIST.RT, T.DIST.2T, TDIST and CONFIDENCE.T beside
  T.DIST and T.INV.2T. They reuse its tail, quantile and df helpers (`lower_tail`, `cdf`, `two_tailed_quantile`,
  `_tail_x`, `bounded_df`), and T.DIST's `x` parameter.
  - T.DIST.RT is `cdf(-x)` and keeps the far tail with no complement. T.DIST.2T and TDIST share one value function,
    tails times the upper tail. TDIST's `tails` is an integer in [1, 2] that truncates (statistics.md T-1048).
  - CONFIDENCE.T is built on T.INV.2T directly: T-4101's `defs/confidence.py` is not on main.
  - Domains, all from evidence. x >= 0 for T.DIST.2T and TDIST. df is in [1, 1E10] for the three tails and truncates
    (T-1043, T-1048). For CONFIDENCE.T, alpha is in (0, 1), sd > 0, and the size is in [2, 1E10 + 1] and truncates
    (T-1043: n = 1 is `#DIV/0!`, n = 10000000002 is `#NUM!`).
  - The huge-x `#NUM!` cut (x² overflow, about 1.34E+154) is not a domain rule: it is unprobed at the exact point, so
    those points keep their math value and Excel's error code is the target.
  - A CONFIDENCE.T point whose sd·t, sd/√n, t/√n or margin leaves the normal double range has no value
    (`NoValueError`, unprobed).
  - Tests: tools/tests/test_accuracy_student_t_tails.py, against Microsoft's examples, the semantics entries' Excel
    values and the evidenced error points.
- Before generating, the definitions ran over the literal one-call cells of eval/suites:
  - T.DIST.RT: 16,950 cells with a value (16,938 both numbers, 11 both errors).
  - T.DIST.2T: 306 (236 numbers, 65 errors). TDIST: 206 (134 numbers, 66 errors). CONFIDENCE.T: 135 (94 numbers, 41
    errors).
  - No cell has a math domain error where Excel gives a number.
  - 12 cells are the reverse, an Excel `#NUM!` against a math number: x = 1E+300 or 1E+307 (gen-m9 and
    probes/t2436/fec39efaba9284c3 A1), the documented x² overflow.
- Tables `eval/accuracy/tables/{T.DIST.RT,T.DIST.2T,TDIST,CONFIDENCE.T}/s1`: 120, 120, 200 and 197 points (3 CONFIDENCE.T
  draws had no value). Filled from excel-graph with 0 failures and 0 table defects.
- `rustsheet-eval accuracy`: 637 new points, 0 failing, 0 table defects. `missing tables` went from 72 to 68.
  - Inside the envelope: 632. T.DIST.RT 103 on both sides and 17 through Excel's 15 digits only; T.DIST.2T 112 and 8;
    TDIST 185 and 15; CONFIDENCE.T 162 both, 7 Excel only, 23 math only.
  - 5 known failures added:
    - CONFIDENCE.T/s1 A112, A123 and A134 (T-4541): alpha 2.2E-308, Excel `#NUM!`.
    - A155 and A163 (T-3363): df 2.3E9 and 6.2E6, where the engine and Excel are both off the math.
  - The engine equals Excel bit for bit on 453 of the 637 points: 378 of 559 numbers and 75 of 78 errors.
- Excel kept as the target (decision 0300): points where Excel is more than 1e-4 relative off the math. None is marked.
  - Table points: CONFIDENCE.T/s1 A139, A147, A154, A178, A180, A182, A183, A187-A191 and A197.
    - Alpha is 1.3E-301 to 9.2E-165, n is 3884 to 7.8E9, and Excel is 0.81 to 1.13 relative off (about twice the
      margin), the tiny-p T.INV.2T gap.
    - Example: `CONFIDENCE.T(3.00330653159963E-179,2400358.77101737,5887)`, Excel 1930850.749, math 924965.593.
    - The engine is at the math, so these pass through the math side.
  - Probe sidecar cells: probes-numeric/t1531-t-tiny-p A84 and A108, `CONFIDENCE.T(1E-300,1,100000)` and
    `CONFIDENCE.T(1E-300,1,100001)`. Excel gives 0.2502, against math 0.11762 (1.13 relative).
  - Probe sidecar cells: probes/t4254-beta-t-l140-zero-cut E1:E22 and F1:F22, and t4254-beta-t-zero-wide F30 (45
    cells).
    - These are `T.DIST.RT` at x 37 to 38 with df 1E5 and 1E7, where Excel gives 0 against a true tail of 9E-300 to
      7E-313.
    - Statistics.md already documents this zero cut in the T-4254 paragraph. T-4437 owns it.
  - No T.DIST.RT, T.DIST.2T or TDIST table point: their only gaps are underflows to 0, where the math is below the
    smallest double.
- Semantics (statistics.md), each limited to the cited cells:
  - "Scalar distributions at extreme arguments, part 2": the table brackets of the huge-x `#NUM!` cut for T.DIST.RT
    (both signs), T.DIST.2T and TDIST.
  - "Student t sweep (T-1043)": the 1E10 + 1 df cut of T.DIST.RT, T.DIST.2T and TDIST.
  - "Legacy distribution names sweep, second family": x = -2.2E-308 is already `#NUM!`.
  - "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308": CONFIDENCE.T at alpha 2.2E-308.
  - "Tiny-p quantiles": the CONFIDENCE.T points above.
  - T-4541's and T-3363's Context name their new points.
- The probes and probes-numeric math sidecars were regenerated with `--function` for the four functions. The change
  only adds cells: 12,803 cells in 13 sidecars, and 7 new sidecars. `make eval-math-check` reports "current".
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
