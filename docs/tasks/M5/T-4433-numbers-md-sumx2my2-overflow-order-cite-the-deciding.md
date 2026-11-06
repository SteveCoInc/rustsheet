---
id: T-4433
title: "numbers.md SUMX2MY2 overflow order: cite the deciding cells (A1, A3 for model b; A5:A6 for the 80-bit square) and re-wrap T-4424's sentence"
milestone: M5
lane: functions-math
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
numbers.md's SUMX2MY2 overflow-order sentence (from T-4424) cites only the probe cells that decide each model, and is
wrapped at about 120 characters like the text around it.

## Context
- The excel-fidelity review of PR #3462 (T-4424, head e0b263171) found two citation slips in the sentence that
  replaced numbers.md's "Unprobed: whether SUMX2MY2's running sum takes x^2 and then -y^2" (section "SUMPRODUCT,
  SUMX2MY2 and SUMX2PY2 overflow at a running sum"):
  - "A1:A2, so not model b". A2 runs the pairs in the order where `(s + x^2) - y^2` stays finite, so it decides
    nothing. The cells that rule out model b are A1 and A3 (probes/t4424-sumx2my2-overflow-order).
  - "A5:A7, so not an 80-bit square". In A7 the first pair's -y^2 is already beyond the double range. The cells that
    decide this are A5:A6.
- The sentence was not re-wrapped, so one line runs to about 170 characters.
- Values to keep: A1:A2 are 1.6899999999999998E+308, A3:A4 are -1.6899999999999998E+308, and A5:A7 are `#NUM!`.
  The oracle sidecar is eval/suites/probes/cases/t4424-sumx2my2-overflow-order.oracle.json.

## Acceptance criteria
- [x] The sentence cites A1 and A3 for "not model b" and A5:A6 for "not an 80-bit square". It may still quote A2,
      A4 and A7 as values, but not as deciding cells.
- [x] The paragraph is re-wrapped at about 120 characters, with no other wording change.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine or probe change: the result itself (model a) stands.

## Outcome
Fixed in T-4424's own PR (#3462) before it merged, so no separate change was needed. The fidelity review of the
first fix found a further slip: A3 does not decide model b either, because it stays finite under it. The sentence now
says that A1 alone rules out x^2-then-(-y^2) and that A2:A4 stay finite in every model, cites A5:A6 for the 80-bit
square, and is wrapped at about 120 characters. The excel-fidelity and design reviewers approved it on #3462's head
616a11698.
