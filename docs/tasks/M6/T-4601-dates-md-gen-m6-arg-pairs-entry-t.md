---
id: T-4601
title: "dates.md gen-m6 arg-pairs entry (T-3495): replace the stale NETWORKDAYS.INTL day-0 claim and probe-pending status with T-3537's scalar-holiday finding"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
dates.md "gen-m6 arg-pairs batch (T-3495): DAYS error order and NETWORKDAYS.INTL on day 0" says what the evidence
shows now. Its NETWORKDAYS.INTL claim and its status come from T-3537's finding, and it duplicates no rule that
another dates.md section already states.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). No unowned visibly-off
  cells were left, so this triage checked the semantics notes against the done tasks they cite.
- The entry, as it stands:
  - It says "NETWORKDAYS.INTL: every failing cell has a start or end on serial day 0".
  - It says "Ours: ... NETWORKDAYS.INTL is one day off in the other direction".
  - Its status is "UNVERIFIED (probe pending; cells are oracle values, the general rule is not probed)".
- T-3537 (done) found that the cause was not day 0. A **scalar** NETWORKDAYS.INTL holiday is dropped only on
  Saturday and Sunday, whatever the weekend argument is. Its probes are networkdays-intl-holiday-forms,
  -weekend-masks, -kinds, -kinds-let and -unary-fn. The rule is in dates.md "NETWORKDAYS.INTL: a scalar holiday
  ignores the weekend argument" (VERIFIED). gen-m6/a4a301d00b46acf3 has no known-failure line in eval/baseline/gen-m6.txt
  (`C ... 174` only).
- The DAYS half is already in dates.md "DAYS error order" (VERIFIED, the same ten cells, T-3536).

## Acceptance criteria
- [ ] The entry no longer says the NETWORKDAYS.INTL cells fail because of day 0, or that ours is one day off. It
      points to "NETWORKDAYS.INTL: a scalar holiday ignores the weekend argument" for the cause (T-3537).
- [ ] The DAYS half is a pointer to "DAYS error order", not a second statement of that rule (one definition per
      concept). The alternative is to fold the entry into the two sections and delete it.
- [ ] No "probe pending" status is left on the entry. It reads VERIFIED with the cases cited, or it is gone.
- [ ] Docs only: nothing changes under crates/, eval/ or tools/. `make check` green; reviewers approve.

## Out of scope
- dates.md "WEEKDAY of serial 0 and return_type 0" and its T-1254 status: T-3306 item 6 owns it.
