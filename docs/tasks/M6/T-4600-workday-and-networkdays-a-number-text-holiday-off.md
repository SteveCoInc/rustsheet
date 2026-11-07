---
id: T-4600
title: "WORKDAY and NETWORKDAYS: a number-text holiday off the calendar (a \"-1\" cell or literal): probe whether it errs before or after the weekend, and match"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells show where WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL put a single holiday that is number
text off the calendar (`"-1"`, `"3000000"`) in their error order: step (4) of dates.md, before the weekend, or step (6),
after it. dates.md states the rule with the cells, and ours matches every cell.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). `scripts/exit_owners.py
  --all-off` lists no unowned visibly-off cell outside the M16 pivot probes, so this triage read the rules the semantics
  docs leave open.
- dates.md "WORKDAY and NETWORKDAYS holidays: blank and number-text cells, and the error order (T-2270, T-2272)", the
  "Ours" paragraph ends: "Unprobed: whether single number text off the calendar (a `"-1"` cell) is step (4) or (6); ours
  (6) (T-2271 probes it)". T-2271 is done, and its Outcome (weekend number coercion) did not probe it.
- The order it refers to: probes/workday-holidays-error-order (T-2272). (4) a single non-number holiday (`#VALUE!`), (5)
  the weekend, all-weekend mask included, (6) any other holidays error, such as the number -1:
  `WORKDAY.INTL(45000,5,"1111111",-1)` is `#VALUE!` (Q45), the weekend's error.
- The same paragraph also says "a cell holding number text is a holiday, counted rather than skipped (... so
  skip-vs-count is UNVERIFIED)". probes/t4397-holiday-text-forms (T-4397, 516 cells, all pass) shows number text in a
  cell acting as the serial it reads as, so that clause is stale. Replace it with the T-4397 citation in this PR.
- Ours (release build of this branch, scratch case): with A1 holding the text "-1", `WORKDAY(45000,5,A1)` and
  `NETWORKDAYS(45000,45010,A1)` are `#NUM!`, and `WORKDAY.INTL(45000,5,"1111111",A1)` and
  `WORKDAY.INTL(45000,5,"1111111","-1")` are `#VALUE!` (step 6: the weekend wins).
- Code: crates/rustsheet-engine/src/functions/date_and_time/holidays.rs (`holiday_serials`, `HolidaysError::before_weekend`),
  `Model::workday_of` and `Model::networkdays_of`.

## Probe
excel-probe skill, excel-graph, tag M6. Name the case probes/t4600-holiday-number-text-off-calendar.
- Holidays: the text `"-1"`, `"3000000"` (past 9999-12-31) and `"-0.5"`, each as a literal, as a cell, and as a
  one-element array `{"-1"}`. Add the number -1 in a cell as the control.
- Calls: WORKDAY(45000,5,h); WORKDAY.INTL(45000,5,1,h), (45000,5,"1111111",h) and (45000,5,99,h); NETWORKDAYS(45000,45010,h);
  NETWORKDAYS.INTL(45000,45010,1,h), (..,"1111111",h) and (..,99,h). Wrap each in ERROR.TYPE beside the bare call.

## Acceptance criteria
- [ ] probes/t4600-holiday-number-text-off-calendar is committed with its excel-graph oracle sidecar.
- [ ] Every probe cell passes, or each visibly-off cell has an owner task.
- [ ] dates.md states which step number text off the calendar takes, citing the cells, and its "Unprobed" sentence and
      the stale skip-vs-count clause are gone (the latter cites probes/t4397-holiday-text-forms).
- [ ] A unit test pins one step-(4)-versus-(6) cell if the engine changes.
- [ ] No new known-failure line in eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Holiday text forms returned by a formula (T-4408) and the weekend argument's own coercion (T-2591).

## Outcome
Probed (probes/t4600-holiday-number-text-off-calendar, 176 cells, excel-graph): number text off the calendar
is error-order step (6), after the weekend, in every form; our engine already matched, all cells pass, no
engine change. dates.md states the rule; the stale "Unprobed" sentence and the skip-vs-count clause are gone
(the latter now cites probes/t4397-holiday-text-forms). Eval delta: +176 probe cells passing, no new known
failures. The case has no Gate A one-call cell, so it needs no `.math.json` sidecar (`make eval-math-check` current).
