---
id: T-3915
title: "Reword the MAX_PARENTHESES and overflowed-reciprocal docs and their semantics notes (T-3870, T-3883 review MINORs)"
milestone: M2
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The `MAX_PARENTHESES` doc comment and the two T-3870 sentences in docs/excel-semantics/syntax.md read correctly. Only the
wording changes, not what they claim.

## Context
- MINOR findings of all three reviewers on PR #2955 (T-3870). They were deferred because every reviewer approved.
- crates/rustsheet-engine/src/expressions/parser/depth.rs, the doc of `MAX_PARENTHESES`: "and so do levels of `-(`" now
  follows the new call clause, so it reads as if `-(` levels were also "not a group". Make the call-parenthesis point
  its own sentence, after the `-(` clause, and keep lines within the usual width.
- docs/excel-semantics/syntax.md, "Formula nesting depth cap": the parenthetical "(T-3870 added `ISBLANK(A1)` ... TRUE
  on all 23 ...)" sits inside the T-3689 sentence, before "so T-3689 restored them". That makes it read as the reason
  for T-3689's restore. Move it to its own sentence after the T-3689 one.
- Also the MINORs of T-3883's review (PR #2964, all reviewers approved):
  - In crates/rustsheet-engine/src/functions/engineering/complex/trig.rs, move the test
    `overflowed_reciprocals_divide_by_cosh` above the "IMCSCH's negated band (T-3881 ...)" comment, so that comment
    sits directly on `imcsch_negates_csch_in_the_band`.
  - `overflowed_reciprocal`'s doc still says "computed as `2e^-|v|`". Say that the work is in `overflowed_parts`:
    parts over cosh |v|, then `2e^-|v|` past cosh's overflow.
  - In docs/excel-semantics/engineering.md "Tiny results" (the T-2335 entry), "i.e. times 2e^-|v|" should read "over
    M". Also note that e^|v|/2, libm cosh and `inverse_exp_sinh_cosh`'s cosh tie on every oracle cell (T-3883's
    Outcome).

## Acceptance criteria
- [ ] Each place above is reworded or moved, and the facts and the cited cases are unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to the parser or to a probe case.
