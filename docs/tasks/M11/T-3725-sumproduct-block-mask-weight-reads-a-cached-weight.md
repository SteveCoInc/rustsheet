---
id: T-3725
title: "SUMPRODUCT block*mask*weight reads a cached weight range without building arrays (calipers pv26_002)"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
SUMPRODUCT's `block*mask*...` argument (T-3230) also folds in operands that are cached numeric
ranges ("weights"): one column as tall as the block, one row as wide, or the block's own shape.
It does so without building the operators' arrays. Values and errors stay the operators'.

## Context
Calipers `pv26_002` has 41 calls of the form
`SUMPRODUCT('Customer Data'!$F$2:$DX$1465*('Customer Data'!$F$1:$DX$1=D$12)*'Customer Data'!$DZ$2:$DZ$1465)`.
The block is 123 x 1,464 cells, times a header mask, times an amount column. `MaskedBlock` took only
masks, so at the amount column every call fell back to the operators. Each call then built three
arrays of 180,000 elements: `arithmetic_on_values` was the largest leaf under `fn_sumproduct`
(2026-11-01 samples).

`*` is `times` = `finite_or_num(extended_product)`, with no flush between products, and a mask
element is 1 or 0. So where every mask is TRUE, an element is the cell's number times each weight's,
each product `extended_product`-rounded; anywhere else it is a zero. The weight's cells are read as
`*` reads a range operand, which is how `BlockNumbers` already reads the block.

## Acceptance criteria
- [x] `MaskedBlock::weigh` takes a cached `BlockNumbers` whose rows are 1 or the block's and whose
      columns are 1 or the block's, and not a single cell. Its first error is placed at its first
      position in the block, row by row, and an earlier operand keeps a tie, as masks do.
- [x] A weight whose products could overflow into `*`'s `#NUM!` is refused, and the operators run.
      `bound`, the product of each factor's largest magnitude, must stay below `f64::MAX / 2`;
      `BlockNumbers::largest` is new.
- [x] Tests compare cached and walked runs, with the pass caches off, for every weight shape and
      position in the chain, two weights, and errors and text in weights. Weight errors are placed
      to compete with each other. Refused cases: overflow (`#NUM!`), a weight not cached yet, one
      cell, and mismatched shapes. A hand-worked sum and a double-rounding product
      (probes-numeric/double-rounding-ops A1) pin the values. Mutating the error position, the
      broadcast, the bound or the rounding fails them.
- [x] calipers `pv26_002` release `evaluate()` before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Building the masks themselves without arrays (`range=key` over a cached range). The 1,575
  `'Quarterly ARR Rollup'` calls still build two comparison arrays each.

## Outcome
`MaskedBlock` gains `Weight`, `weigh` and `bound`, `Model::weigh_masked` reads a range operand as a
weight, and `BlockNumbers` records its largest magnitude. All 41 `Customer Data` calls of `pv26_002`
now read masked blocks. Calipers `pv26_002` release `evaluate()`, interleaved runs: 7.0–7.3 s before,
5.8–6.2 s after. Re-measured on main at 2026-11-07, which is faster in the meantime: 4.0–4.6 s before, 3.98–4.15 s after.
