---
id: T-4408
title: "WORKDAY and NETWORKDAYS holiday given as text returned by a formula directly as the argument: probe it"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells show how WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL read a holiday given as text that a
formula returns directly in the argument, such as `WORKDAY(45000,5,TEXT(45001,"0"))`. dates.md records the rule, and
ours matches every cell.

## Context
- This is a follow-up from T-4397 (PR #3427): a MINOR finding from the fidelity reviewer. probes/t4397-holiday-text-forms
  covers a text literal, a cell, a one-element array and a two-cell range. probes/t2369-workday-text-holidays C33:C44
  covers formula text only through a cell (A9:A11). No cell gives text returned by a formula, such as `TEXT(..)`,
  `45001&""` or `IF(TRUE,"3/16/2023")`, directly as the holidays argument.
- dates.md "WORKDAY and NETWORKDAYS holidays: blank and number-text cells, and the error order (T-2270, T-2272)", the
  "Ours" paragraph. dates.md "NETWORKDAYS.INTL: a scalar holiday ignores the weekend argument" lists `C1&""` as a
  scalar from its number form only.
- Code: crates/rustsheet-engine/src/functions/date_and_time/holidays.rs (`holiday_serials`, `Holidays::scalar`).
- Probe design: reuse `docs/research/t4397/gen_probe.py`'s layout. Start at 45000 (Wed). Use a Thursday (45001) and a
  Saturday (45003) holiday. Give the holiday as `TEXT(d,"0")`, `TEXT(d,"m/d/yyyy")`, `d&""` and
  `IF(TRUE,"<date text>")`. Test all six function forms of T-4397 (WORKDAY.INTL and NETWORKDAYS.INTL with weekend 1
  and 13).

## Acceptance criteria
- [ ] A probe case `probes/t4408-holiday-formula-text` is committed with its excel-graph oracle sidecar.
- [ ] Every probe cell passes, or each visibly-off cell has an owner task.
- [ ] dates.md states the rule for formula-returned text and cites the cells. That includes whether it is a scalar
      holiday for NETWORKDAYS.INTL's weekend rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Text literals, cells, arrays and ranges (T-4397).

## Outcome
Probe `probes/t4408-holiday-formula-text` (132 cells, excel-graph; generator docs/research/t4408/gen_probe.py): formula-returned
holiday text acts as the serial it reads as, and as a scalar holiday for NETWORKDAYS.INTL's weekend rule. Ours passes
all 132 cells; no code change. Rule recorded in dates.md. No new known failures; no follow-ups.
