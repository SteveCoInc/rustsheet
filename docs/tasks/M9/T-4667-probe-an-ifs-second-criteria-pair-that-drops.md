---
id: T-4667
title: "Probe an IFS second criteria pair that drops an error row, and a #NUM! value-range error under AVERAGEIF(S) (T-4606 review follow-up)"
milestone: M9
lane: functions-criteria
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The per-position error rule for AVERAGEIFS, AVERAGEIF, MAXIFS, MINIFS and SUMIFS with an array criterion over a value
range that holds errors (docs/excel-semantics/criteria.md, from T-4606) has evidence for its two unprobed corners.

## Context
- T-4606's review (excel-fidelity, PR #3657) found two MINOR gaps in the evidence of
  probes/t4606-ifs-array-criterion-value-error:
  - the rule says the error is the result only at the positions whose selected rows include it, but no probe has a
    second criteria pair that drops an error row the first criterion selects (B1:B4 = 1, 1, 0, 1 drops only row 3,
    and both error rows stay selected);
  - for AVERAGEIFS and AVERAGEIF, `{"c"}` gives #DIV/0!, which the probe's #DIV/0! value-range error cannot tell
    apart from the no-numeric-match #DIV/0!.
- The criteria.md sentence "all five functions (MINIFS too)" follows a list of six; name the five probed ones.

## Acceptance criteria
- [ ] A probe case (excel-probe skill) with a second pair that excludes an error row (e.g. B1 = 0), and one with a
      #NUM! value-range error where the no-match answer is #DIV/0!, for the five functions; oracle values recorded.
- [ ] criteria.md states the rule with those cells and the wording fix; any mismatch filed as its own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which error wins when one position selects two different errors (left unprobed by T-4606, its own question).

## Outcome
Probe `t4667-ifs-second-pair-and-num-value-error` filled on excel-graph: a second pair that drops an error row removes the
error at that position (7), and a #NUM! value-range error at "c" stays #NUM! for all five functions (not #DIV/0!). The rule
in criteria.md stands; the wording fix is made. Eval delta: +1 probe case (76 cells), 0 fixed, 0 regressions, baseline +1 line.
