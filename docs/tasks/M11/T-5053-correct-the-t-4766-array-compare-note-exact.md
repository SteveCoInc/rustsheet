---
id: T-5053
title: "Correct the T-4766 array-compare note: exact-boundary evidence, the 15-digit literal cut, unprobed edges"
milestone: M11
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4766]
---

## Goal
The T-4766 entry in docs/excel-semantics/numbers.md ("Worksheet EXP (T-1931)", the array-comparison paragraph) and the T-4766
Outcome state the evidence exactly: the exact boundary is probed, the E5/E6 cause is named, and the unprobed edges are labelled.

## Context
From the excel-fidelity review of PR #4036 (T-4766), MINOR findings, docs only:
- probes/t4766-array-subnormal-fresh S!A2 builds exactly 2^48 * 2^-1074 = 2^-1026 (its multiplier 281474976710656 has 15
  digits, so it is not cut) and Excel gives `(...+{0})=0` FALSE there, while A1 (2^48-1) is TRUE. So the strict `<` is pinned to
  the ulp; the sentence "the value exactly at 2^-1026 ... not separately probed" is wrong.
- Fresh E4:E6 are TRUE because the 16-digit literal 1125899906842624 is cut to 15 significant digits (1125899906842620), so the
  gaps are 2^48-5, 2^48-4 and 2^48-3 units. The note's "reproduces Excel's arithmetic for building the operands" should say so.
- Unprobed, the engine's choice: two normal numbers less than 2^-1026 apart (below about 2^-974), and an empty range element
  read as 0 against a tiny number (`within_array_compare_gap` fires only for Number/Number).
- The constant 2^-1026 is an oracle-measured boundary pinned by fresh A1/A2 and the -order bracket, not a decision 0188 R4 fit;
  the Outcome's "Fit guards" paragraph calls the training cells "(the hold-out)" and should be reworded.

## Acceptance criteria
- [ ] numbers.md's T-4766 paragraph cites fresh A1/A2 for the exact boundary, names the 15-digit literal cut for E4:E6, and marks
      the two unprobed edges as the engine's choice; the code comment of `arithmetic::within_array_compare_gap` agrees.
- [ ] The T-4766 task file's Outcome calls 2^-1026 a measured boundary (no R4 fit) and drops the "(the hold-out)" wording.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing the two unprobed edges (file a probe task if wanted).

## Outcome
Docs and a code comment only. numbers.md's T-4766 paragraph now cites fresh A1/A2 for the exact boundary, names the 15-digit
literal cut behind E4:E6 and marks the two unprobed edges as the engine's choice; the `within_array_compare_gap` comment agrees;
the T-4766 Outcome calls 2^-1026 a measured boundary, not an R4 fit. Eval delta: none.
