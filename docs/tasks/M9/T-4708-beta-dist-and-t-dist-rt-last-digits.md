---
id: T-4708
title: "BETA.DIST last digits in the t4254 deep-tail probes at a = 1E6 to 5E6: the 48 cells within the 13-digit bar that no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4707]
defer: phase-1b
---

## Goal
The 48 cells below match Excel to the bit, or the Outcome records, per case, the bit-exact count and why the rest
cannot be derived. They are the BETA.DIST cells of the t4254 deep-tail probes (tails 1E-170 to 1E-291, a = 1E6 to 5E6,
b = 3.5 to 10.5) that are already within the incomplete-beta family's 13-digit bar.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/functions-lookup/T-4570-xlookup-ref-element, based on origin/main
  5aed684f1). I checked every failing cell of report.json against open tasks with `scripts/exit_owners.py`'s `owned`
  rule. In probes/t4254-beta-t-l140-zero-cut and probes/t4254-beta-t-zero-wide, T-4564 owns the zeros and the
  8th/9th-digit near-edge numbers, and T-4707 owns the 9 visibly-off numbers. These are what is left. All are
  BETA.DIST, and none is visibly off under decisions 0114 and 0267: 30 `residue13` and 18 one-ulp. By the first wrong
  significant digit:
  - probes/t4254-beta-t-l140-zero-cut.case.json (17; (a, b) = (2E6, 10.5) in A, (2E6, 10) in C):
    - 14th digit: A1, A3, A4, A5, A8, A10, C2, C3, C4, C5, C7, C9, C10;
    - 15th digit: A6, A11, C1, C8.
  - probes/t4254-beta-t-zero-wide.case.json (31; (5E6, 3.5) in A, (5E6, 4) in B, (1E6, 3.5) in C, n = 3 bup terms):
    - 14th digit: A26, A27, B26, B27, C26, C27, C29;
    - 15th digit: A28, C28;
    - one ulp: A3, A5, A6, A8, A13, A15, A25, B11, B14, B18, B20, B23, B24, B25, C2, C3, C4, C6, C10, C11, C18, C24.
- Examples (Excel vs ours, `make eval-show CASE=...`; math from the eval sidecar):
  - l140-zero-cut A4 `BETA.DIST(0.999647862331594,2000000,10.5,TRUE)`: Excel 1.225623763122006E-285, ours
    1.2256237631218695E-285. Excel is 1.1E-13 off the math and ours 1.5E-16.
  - zero-wide A27 `BETA.DIST(0.999867334917779,5000000,3.5,TRUE)`: Excel 2.7333896131375853E-282, ours
    2.7333896131378426E-282. Excel is 9.4E-14 off the math and ours 2.8E-16.
  - zero-wide A13 `BETA.DIST(0.99989535531834,5000000,3.5,TRUE)`: Excel 1.0771810924473524E-221, ours
    1.0771810924473523E-221 (one ulp). Both are 3E-14 off the math, so here ours already follows Excel's formulation
    to the last rounding.
- On the 14th-digit cells Excel is 1E-14 to 1.3E-13 off the 50-digit math, and ours is mostly within 1E-15. This is
  the noise decision 0267 describes, so matching it means reproducing Excel's arithmetic. Decision 0267 item 4: Tier C
  and D get no new work of their own. Hence `defer: phase-1b`, and the dependency on T-4707, which works the same
  kernels on the Tier B cells of these cases. Re-measure after it lands.
- Kernels: `bratio/bup.rs` `l140_bup_bgrat` (MAX_BUP_TERMS 9; the (2E6, 10) and (2E6, 10.5) cells take the fallback
  kernel past it) and the n = 3 bup path with bup's register-rounded W (T-3382, T-3770).

## Acceptance criteria
- [ ] Each cell above is bit-exact, or the Outcome lists it with its Excel, ours and 50-digit values and the reason it
      stays. These known-failure lines disappear from eval/baseline/probes.txt with no new ones.
- [ ] Decision 0188 R4 guards (reviewer-excel-fidelity check 7), for any formulation chosen by scoring:
  - hold-out counts (exact / 4 ulp / 15 digits) on the sweep's hold-out, not on cells used to choose;
  - a fresh sweep, drawn after the formulation is frozen, is scored;
  - the Gate A envelope result is in the Outcome (within 15 digits of Excel, or within 1E-9 of the 50-digit value),
    listing every point outside;
  - the code carries a `FITTED` tag with the fitted range;
  - the reproducibility floor is met;
  - there is no branch that keeps already-matched cells by special case.
- [ ] No gated or probes-numeric cell goes from within 15 digits to visibly off (decision 0267 item 6 governs any
      family regression to the 14th digit).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells of these cases (T-4707), and their zeros and near-edge numbers (T-4564).
- The t4456 deep-tail cells (T-4463, T-4577).
