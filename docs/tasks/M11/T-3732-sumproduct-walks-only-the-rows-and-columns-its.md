---
id: T-3732
title: "SUMPRODUCT walks only the rows and columns its masked arguments keep (calipers pv26_002)"
milestone: M11
lane: perf
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
SUMPRODUCT with a `block*mask*...` argument (T-3230) costs what its masks keep, not the block's size.
The total is bit for bit the same.

## Context
After T-3725, `fn_sumproduct`'s own summation loop was its largest cost on calipers `pv26_002`. It ran
over every element of the shape and folded every factor, and 1,575 calls each multiply a 1,440 x 41
block by a header mask and a key mask: about 93M element products, nearly all of them masked-off zeros.
A masked argument's element is a zero wherever one of its masks is FALSE, so the product there is a
zero. Adding a zero leaves the running total as it is: `extended_sum` adds a zero plainly, and the
total is never -0. The tail of an all-tailed call already relies on this.

## Acceptance criteria
- [x] `fn_sumproduct` walks only the rows and columns that every masked argument has on, in the
      same row-by-row order. With no masked argument, every row and column is walked as before.
- [x] The existing masked SUMPRODUCT tests (`test_sumproduct_masked.rs`: two masked arguments with
      different masks, masked beside grid arguments, errors) pass. Mutating the "every masked
      argument" rule to "any", or shifting the row test, fails them.
- [x] calipers `pv26_002` release `evaluate()` before and after, in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Building the masks without arrays (`range=key` comparisons over cached ranges).

## Outcome
`fn_sumproduct` collects the masked arguments, walks the columns all of them keep once, and skips
rows any of them masks off. Calipers `pv26_002` release `evaluate()`, interleaved runs on a noisy
machine: 6.5–7.2 s before (T-3725), 5.8–6.6 s after. Re-measured on main at 2026-11-07 (with T-3725 merged): 4.18–4.64 s before, 3.66–4.53 s after, faster in each of five interleaved runs.
