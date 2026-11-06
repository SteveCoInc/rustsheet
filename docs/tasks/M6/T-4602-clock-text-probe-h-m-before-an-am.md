---
id: T-4602
title: "Clock text: probe H:M: before an AM/PM suffix (\"10:30: PM\") and a tab or no-break space inside H:M.f:S or between a date and its clock, and match"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells decide three time-text shapes that dates.md "Time text" leaves unprobed, and ours matches every cell:
- `H:M:` followed by an AM/PM suffix (`"10:30: PM"`);
- a tab, no-break space (U+00A0) or ideographic space (U+3000) inside the fraction of `H:M.f:S` text;
- the same characters between the date and the clock of date-clock text.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). No unowned visibly-off
  cells were left, so this triage read the rules the semantics docs mark unprobed.
- dates.md "Time text":
  - The trailing-colon bullet (probes/t2352-time-text-trailing-colon, T-2352) ends: "Unprobed: `H:M:` before an AM/PM
    suffix (`"10:30: PM"`; the engine reads it as 10:30 PM)". The same probe gives `"12:PM"` and `"12: PM"` (a bare
    hour, `:`, then AM/PM) as `#VALUE!`, so the `H:M:` form may differ from ours.
  - The space bullet (value-clock-text rows 38, 42-45) ends: "Unprobed: such characters inside the fraction of
    `H:M.f:S` text (inferred from this rule) and between the date and the clock of date-clock text."
- Ours (release build of this branch, scratch case):
  - `VALUE("10:30: PM")`, `TIMEVALUE("10:30: PM")` and `0+"10:30: PM"` are 0.9375;
  - `VALUE("10:30:45: PM")` is 0.9380208333333333 and `VALUE("10:30: AM")` 0.4375;
  - `VALUE("10:30:PM")` is `#VALUE!`;
  - `VALUE("10:11.5<U+00A0>:12")`, `VALUE("10:11:12.<U+00A0>5")`, `VALUE("1/2/2020<TAB>10:00")` and
    `VALUE("1/2/2020<U+00A0>10:00")` are `#VALUE!`;
  - `VALUE("1/2/2020  10:00")` (two spaces) is 43832.416666666664.
- Code: `parse_clock_text`, `colon_tail_parts`, `parse_simple_am_pm` and `parse_date_clock_parts` in
  crates/rustsheet-engine/src/formatter/date_clock.rs.

## Probe
excel-probe skill, excel-graph, tag M6. Name the case probes/t4602-clock-text-colon-am-pm-and-spaces. Write the
non-ASCII characters with CHAR/UNICHAR (`"10:11.5"&UNICHAR(160)&":12"`), so the formula text stays ASCII.
- AM/PM after a trailing colon: `"10:30: PM"`, `"10:30:PM"`, `"10:30:  PM"`, `"10:30:45: PM"`, `"10:30: AM"`,
  `"12:00: AM"`, `"13:30: PM"`, `"1/2/2020 10:30: PM"`.
- Odd characters: TAB, U+00A0 and U+3000 before, inside and after the fraction of `"10:11:12.5"` and `"10:11.5:12"`.
  Then the same three between `"1/2/2020"` and `"10:00"`, and two ordinary spaces there as the control.
- Read each one through VALUE, TIMEVALUE and `0+text`, and DATEVALUE for the date-clock forms.

## Acceptance criteria
- [ ] probes/t4602-clock-text-colon-am-pm-and-spaces is committed with its excel-graph oracle sidecar.
- [ ] Every probe cell passes, or each visibly-off cell has an owner task.
- [ ] dates.md "Time text" states the three rules with the cells, and the two "Unprobed" sentences above are gone.
- [ ] If the engine changes, unit tests pin one cell of each rule, and probes/t2352-time-text-trailing-colon and
      probes/value-clock-text still pass.
- [ ] No new known-failure line in eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed cell input of these forms (number formats on entry), and locales other than en-US.
