---
id: T-2607
title: "Lifted quantiles: an element that refuses its arguments before the search is charged 1, so BETAINV over a 4x51638 array is #SPILL!"
milestone: M9
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A lifted BETA.INV (and the other `QUANTILE_WEIGHT` quantiles where it holds) charges an element 1 when the kernel refuses it
before the search (an error argument, a probability outside (0, 1), alpha or beta <= 0), so a large lift of refused elements
is not `#ERROR!` from the cost cap.

## Context
gen-m9/4250f880ab640c49 Calc!A237, `=BETAINV(1+2^-52,(-1),..,MAKEARRAY(4,DATE(1952,..),..),NORM.S.INV(1.5))`, is a 4 x 51,638
array in Excel, `#SPILL!` in a cell. Since T-2600 the MAKEARRAY is built and the lifting broadcast takes it, but every
element is charged `QUANTILE_WEIGHT` (2^18, lift/kinds/statistical.rs), which caps a lift at 512 elements, so ours is
`#ERROR!`. Here every element is refused before `beta_quantile` runs (`Model::beta_inv_of`: probability 1+2^-52 and alpha
-1 are `#NUM!`, the last argument is an error). A `KernelWork::ByValue` charge (lift/kinds/mod.rs `ValueWork`, as DB/VDB's
`Loop` and the coupon walks do) can see that from the element's values. arrays.md "Arrays past the grid".

- The probe copy of that formula fails the same way (T-2940): probes/t2436/4250f880ab640c49 Calc!A121, T121, U121
  (Excel `#SPILL!`, ROWS 4, COLUMNS 51638; ours `#ERROR!`).

## Acceptance criteria
- [x] gen-m9/4250f880ab640c49 Calc!A237 passes and leaves eval/baseline.
- [x] A lift of refused elements past 512 runs; one of valid elements past the cap is still `#ERROR!` (unit tests).
- [x] `make check` green; reviewers approve.

## Out of scope
- Raising the cost cap.

## Outcome
- **Change:** BETA.INV and BETAINV are charged by value (`KernelWork::ByValue(ValueWork::BetaQuantile)`,
  lift/kinds/mod.rs). An element that `Model::beta_inv_of` refuses before its search costs 1 unit, not
  `QUANTILE_WEIGHT`. An element is refused when it has an error in any argument, a probability outside (0, 1), an
  alpha or beta of 0 or less, or B <= A.
- **Reading of argument values:** only values that read as known numbers are judged. A logical reads as 1 or 0 and a
  blank as 0. A text may read as any number, so it is never treated as refused. A searched element is still charged
  the flat weight, so the cap on real searches is unchanged.
- **Unit test:** `beta_inv_refused_elements_are_charged_one` (lift/tests/kernel_work.rs) covers:
  - 4 x 51,638 refused elements (probability past 1 or alpha below 0, an error, or B <= A) are charged 1 each;
  - a logical probability is refused;
  - a text probability is not refused, and 513 searched elements still pass the cap.
- **Eval:**
  - gen-m9/4250f880ab640c49 Calc!A237 now passes (`#SPILL!`) and leaves eval/baseline.
  - probes/t2436/4250f880ab640c49 A121, T121 and U121 pass (ROWS 4, COLUMNS 51638).
  - The case's other 10 known failures are unchanged.
- **Scope:** F.INV and the T quantiles keep their `Weighted` charge. The task named BETA.INV, and their refusal rules
  differ, so they are not covered here.
- **Docs:** arrays.md "Arrays past the grid" is updated.
