---
id: T-4103
title: "Gate A tables, F aliases: F.DIST.RT, F.INV.RT, FDIST and FINV"
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
F.DIST.RT, F.INV.RT, FDIST and FINV each get a cited mpmath Gate A definition and a committed, oracle-filled `s1` table, and `make check` runs them.
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
- Reuse `defs/fisher.py` (T-3358). FDIST and FINV are right-tailed, like F.DIST.RT and F.INV.RT.
- Sources (statistics.md): "F density is not the beta density, R's df or a GAMMALN formula (T-2513)", "Incomplete beta is
  TOMS 708 `bratio`: one kernel behind BETA.DIST, F, T, BINOM and NEGBINOM (T-2464)", "F.INV.RT saturates at
  3.696066238512854E+150/df1 in the far tail (T-2435)". T-3592 owns 36 F.DIST.RT cells inside Gate A.

## Acceptance criteria
- [x] One definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, reusing the existing definitions,
      each citing its sources, with unit tests against Microsoft's published examples or the semantics entries' values.
- [x] A committed, oracle-filled `s1` table per function; `rustsheet-eval accuracy` reports `missing tables` lower by
      their number. Every point outside the envelope has a `known-failures.tsv` owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other groups of decision 0272.

## Outcome
- `tools/src/rustsheet_tools/accuracy/defs/fisher.py`: F.DIST.RT and FDIST come from one builder, F.INV.RT and FINV
  from another, on the module's existing `upper_tail`, `lower_tail`, `_tail_x` and `_df` (no copy). The new
  `upper_quantile` solves P(F > x) = q on the smaller tail without forming 1 - q. The legacy names differ from the
  modern ones only in the truncation evidence each cites: no legacy-only domain difference is evidenced (T-1047).
  - Domain: x >= 0, df in [1, 1E10] truncated; the inverses take p in (0, 1] with p = 1 at 0 (T-1049, T-1047, T-1530).
    The right tails add three evidenced rules, each probed on both names: `#DIV/0!` where fl(df1·x) overflows (T-1290),
    `#NUM!` where fl(df2/fl(df1·x)) does (T-2435; a subnormal x reads as 0), and `#NUM!` where bratio's L140 bup
    running product overflows (`bup_ratio_overflows`, T-4030, T-4044, T-4080). The last is the log-product form the
    entry gives, without TOMS's early break.
  - No unprobed region needed a `NoValueError`.
  - Check over the literal one-call cells of the four functions in eval/suites (235 cases, excel-graph oracle):
    10,098 number/number and 362 error/error cells, 0 where the math is an error and Excel a number or the reverse
    (before the subnormal-x and bup rules: 16). 10 cells have no single value (F.INV.RT/FINV beyond the double range,
    Excel's df2 < 2 saturation, T-2808); 51 were dropped as not converged.
  - Tests: tools/tests/test_accuracy_f_right_tail.py (Microsoft's examples, Excel's values from the entries, the
    legacy names sharing value and domain, the boundary and overflow rules, the bup band).
- Tables `eval/accuracy/tables/{F.DIST.RT,FDIST,F.INV.RT,FINV}/s1`: 196, 197, 174 and 174 points, filled from
  excel-graph with 0 failures. `missing tables` 72 to 68 at the fill; 17 after merging main (21 there without them).
- `rustsheet-eval accuracy`: 741 new points, 738 inside the envelope, 3 known failures, 0 failing, 0 table defects.
  Engine == Excel bit for bit: F.DIST.RT 146, FDIST 154, F.INV.RT 25, FINV 29 (354 of 741).
- Known failures added: accuracy/F.INV.RT/s1 A155 (T-1871: the engine 1.8% off where Excel is the math), A112 and A142
  (T-4568, filed here: Excel 0.9% below the math at p = DBL_MIN, and Excel `#NUM!` at p 1.5E-247; the engine returns
  the math at both).
- **Excel kept as the target (decision 0300)**, no R3 marks. Points more than 1e-4 relative off the math:
  - Tables (19): F.INV.RT/s1 A112, A128, A142 (`#NUM!`), A144, A147, A162, A166, A167, A170, A174; FINV/s1 A112, A128,
    A149, A151, A152, A154, A168, A172, A174. Up to a factor of 31,800 at `F.INV.RT(1.9170293168771E-270,8,78611816)`
    and 7E+19 at `FINV(1.21859163444956E-258,36,3)`. Recorded in statistics.md "F.INV.RT and FINV at a tiny
    probability: Excel's own search, far from the math (T-4103)". F.DIST.RT and FDIST have none.
  - Probe sidecar cells (244, F.DIST.RT and FDIST). Excel is exactly 0 at 229 of them:
    - 2 are correct underflow (probes-numeric t4045-f-dist-rt-zero-cut B54, B55).
    - 197 are T-4045's L140 zero cut at df1 20 to 79: probes t4045-f-dist-rt-zero-cut-fresh 51, probes-numeric
      t4045-f-dist-rt-zero-cut 135, t4017-f-huge-df2-above-mean-* 11.
    - 30 are df1 = 2 zeros of Excel's closed form, still unexplained (t3764-shape-one-complement-fresh 24, t4017 6;
      decision 0267, T-4053).
    The other 15 are df1 = 2 cells of probes-numeric t3764-shape-one-complement-fresh (B144, B272, B294, B305, B328,
    B329, B371, B440, B462, B466, B520, B538, B558, B578, B594), where Excel's closed form cancels to a multiple of
    2^-53 (1.6E-4 to 0.62 relative; decision 0267, T-4053).
  - The F.INV.RT cells of Excel's df2 < 2 saturation (probes t2435-dist-extremes A77, A79, A115-A118;
    t2436/abc2939ebd1e2365 A1) have no math value: the quantile is beyond the double range.
- Semantics: new entry "F.INV.RT and FINV at a tiny probability" (above); the T-1049 entry now states that F.DIST.RT,
  F.INV.RT and FINV truncate df (fdistrt A17, finvrt A22, m9-legacy-finv A69); T-1290's overflow entry cites
  F.DIST.RT/s1 A195 (`#DIV/0!` at df1 9006, x 4.0E+307).
- Math sidecars: `rustsheet-accuracy math --function` for the four names over probes and probes-numeric; only cells
  added.
- Eval delta: none in the gated suites (tools, tables, sidecars and docs only).
