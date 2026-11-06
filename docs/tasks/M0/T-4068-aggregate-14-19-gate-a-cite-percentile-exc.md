---
id: T-4068
title: "AGGREGATE 14-19 Gate A: cite PERCENTILE.EXC's domain and floor(quart); tiny-k region"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-3693]
---

## Goal
The AGGREGATE 14-19 Gate A definition (`tools/src/rustsheet_tools/accuracy/defs/aggregate_array.py`, T-3693) cites
an excel-semantics entry for every rule it uses, and its docstring matches its code.

## Context
Follow-up of T-3693's review (#3118, reviewer-excel-fidelity, MINOR findings):
- Two rules have no `docs/excel-semantics/` entry: PERCENTILE.EXC's domain (k in (0, 1) and the rank k(n+1) in
  [1, n], else `#NUM!`), and QUARTILE.INC/.EXC reading quart as floor(k). `eval/accuracy/tables/AGGREGATE/s1` now
  evidences both: A122, A124, A129 and A137 are `#NUM!`; A113, A118, A121 and A126 (n = 1, k = 0.5) return the value;
  A158 and A160 (QUARTILE.INC, k = 4.5) return the maximum; A25, A29, A56 and A177 (QUARTILE.EXC, k = 3.5) are valid.
- The module doc says a function_num outside 14-19 is a domain error. The `Interval(14, 19)` parameter check does
  raise `DomainError`, but `_value`'s fallback raises a plain `ValueError`, and that branch cannot be reached.
- The draws keep PERCENTILE.INC's k at or above 1e-6. Excel's one-based rank 1 + k(n-1) (statistics.md, VERIFIED)
  loses a smaller k, so where a[1] is near 0, Excel can be far from the math. The table does not cover that region.

## Acceptance criteria
- [x] A statistics.md entry (or a sentence in "PERCENTILE and PERCENTILE.INC interpolate from a one-based rank") states
      PERCENTILE.EXC's domain and the floor(k) quart, citing the AGGREGATE/s1 cells above, and the definition's
      `sources` cites it.
- [x] `_value`'s fallback raises `DomainError` or is documented as unreachable.
- [x] A note in the task Outcome on whether the tiny-k region needs a decision 0188 R3 look (a probe of a few cells is
      enough to decide).
- [x] `make check` green; reviewers approve.

## Out of scope
- Regenerating the s1 table: the definition's values do not change.

## Outcome
Done in T-4092's PR (#3515).
- **New entry:** statistics.md "PERCENTILE.EXC's k and rank domain; QUARTILE floors a fractional quart (T-4092)",
  VERIFIED (excel-graph).
  - It states PERCENTILE.EXC's domain (0 < k < 1 and the rank k(n + 1) in [1, n], else `#NUM!`) and the floored
    quart.
  - It cites the AGGREGATE/s1 cells listed above, probes/stat-order-floor-batch A85, A88 and A89 (`QUARTILE(..,-0.5)`
    is `#NUM!`, so the quart is floored, not truncated), and the PERCENTILE.EXC/s1 and QUARTILE.EXC/s1 cells.
  - AGGREGATE's definition (`aggregate_array.py`) cites the entry in its `sources` and module doc.
- **The `_value` fallback** now raises `DomainError`, with a comment that `evaluate` never reaches it because the
  `Interval(14, 19)` check raises first. The AGGREGATE table regenerates byte-identical.
- **Tiny k:**
  - PERCENTILE/s1 A76 and PERCENTILE.INC/s1 A76 and A81 draw k = 1E-20 over data whose smallest value is 0. Excel
    gives 0, the math a tiny positive value, and the engine matches Excel.
  - Decision 0300 keeps Excel as the target for such points, so the region needs no R3 look.
