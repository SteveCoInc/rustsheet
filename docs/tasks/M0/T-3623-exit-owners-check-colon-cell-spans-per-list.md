---
id: T-3623
title: "exit_owners: check colon cell spans per list block, not per line"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`scripts/exit_owners.py` counts a colon cell span such as `A153:A166` as owned when it sits on a continuation line of
a list item that names the case, as it already does for single A1 names, so wrapped cell lists stop showing as
unowned.

## Context
- The gap triage of 2026-11-01 found this. `names_in_span` (scripts/exit_owners.py) checks spans one line at a time:
  a span only counts when the case name is on the same physical line. `names_beside_case` reads whole blocks, meaning
  a line and its continuation lines (`blocks()`, decision 0159).
- The result: `make exit-owners MILESTONE=M9` reports about 304 t3354 F.DIST cells as unowned, though T-3479 lists
  them in spans on the wrapped lines of its cell list. A triager is then likely to file duplicate tasks for them.
- Keep the reason the check is per line, not per file: a task that quotes a formula range (`Data!B6:C10`) must not
  own those cells. Per block keeps that property.

## Acceptance criteria
- [x] `names_in_span` checks each block from `blocks()` that names the case, not each line.
- [x] A unit test covers both sides: a span on the continuation line of a list item naming the case is owned, and a
      span in a different block (for example a quoted formula range in another paragraph) is not.
- [x] `make exit-owners MILESTONE=M9` no longer reports the t3354-f-dist cells that T-3479 lists. Record the before and
      after counts in the Outcome.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any other ownership rule.

## Outcome
`names_in_span` now checks each block from `blocks()` that names the case, not each physical line. Two
unit tests cover both sides (a span on a continuation line is owned; a quoted range in another block is
not). `make exit-owners MILESTONE=M9` on a fresh `make eval` reports 266 unowned cells before and after,
and no t3354-f-dist cell appears in either run, so the 304-cell symptom from the 2026-11-02 triage
could not be reproduced on this report (those cells no longer fail, or T-3479's list changed); the unit
tests are the evidence for the fix. No eval delta.

## Handoff (resumed)
Implemented and reviewed (design and security APPROVE on 2ca5894; PR #2841). The change is done: `names_in_span`
iterates `blocks()` and two unit tests cover both sides. CI `hygiene` fails on governance:
"protected paths changed without a decision record in docs/decisions/: scripts/exit_owners.py,
scripts/test_exit_owners.py" (run 41492397079). A sonnet run may not add a decision record, so next:
create one with `make new-decision TITLE="..."` (cite decision 0159 and the per-block rule for spans), push,
re-run the reviewers the change concerns, then enqueue. Also note the Outcome: the 304-cell t3354-f-dist
symptom did not reproduce on a fresh `make eval` (266 unowned before and after); the Outcome says `status: done`
text, reset to todo here until the decision lands.
