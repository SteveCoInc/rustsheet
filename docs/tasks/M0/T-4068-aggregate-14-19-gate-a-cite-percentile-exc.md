---
id: T-4068
title: "AGGREGATE 14-19 Gate A: cite PERCENTILE.EXC's domain and floor(quart); tiny-k region"
milestone: M0
lane: harness
status: todo
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
- [ ] A statistics.md entry (or a sentence in "PERCENTILE and PERCENTILE.INC interpolate from a one-based rank") states
      PERCENTILE.EXC's domain and the floor(k) quart, citing the AGGREGATE/s1 cells above, and the definition's
      `sources` cites it.
- [ ] `_value`'s fallback raises `DomainError` or is documented as unreachable.
- [ ] A note in the task Outcome on whether the tiny-k region needs a decision 0188 R3 look (a probe of a few cells is
      enough to decide).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Regenerating the s1 table: the definition's values do not change.
