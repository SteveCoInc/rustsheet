---
id: T-3919
title: "Date text: a day, a slash or hyphen, then a month glued to a one- or two-digit year (1-Jul24, 1/Jul24, 1-Jul5) is a date"
milestone: M2
lane: functions-date
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Date text of a day, a slash or hyphen, then a month name glued to a one- or two-digit year
(`"1-Jul24"`, `"1/Jul24"`, `"1-Jul5"`) is the date Excel reads, in DATEVALUE, VALUE, `0+`, MONTH/DAY,
criteria and typed input, as the four-digit `"1-Jul2024"` and `"1/Jul2024"` already are.

## Context
- Evidence: probes/t3299-date-text-glued-month-3 (excel-graph, 2026-11-03), docs/excel-semantics/dates.md
  "Date text with a month name glued to digits", the T-3299 paragraph. `"1-Jul24"`, `"1-July24"`,
  `"1-jul24"`, `"1-JUL24"`, `"01-Jul24"` are 2024-07-01, `"31-Jul24"` 2024-07-31, `"1-Jul05"` and
  `"1-Jul5"` 2005-07-01, `"1/Jul24"` 2024-07-01; typed, `"1-Jul24"` and `"1/Jul24"` are 45474 with
  format `d-mmm-yy` (docs/research/t3299-glued-month-typed-input-2026-11-03.md). A one- or two-digit
  glued year is the two-digit year rule `"1Jul5"` already follows (2005).
- The engine reads these as `#VALUE!` and keeps them text when typed. Code: `split_glued_month` and
  `parse_separated` in crates/rustsheet-engine/src/formatter/date_text.rs (`parse_date`). The four-digit
  forms and `"1Jul5"` already pass, so the gap is likely where a separated word's last part is a month
  glued to fewer than four digits.
- Do not accept a period or a comma: `"1.Jul24"`, `"1,Jul24"`, `"1.Jul2024"`, `"1,Jul2024"` are
  `#VALUE!` and stay text when typed (rows 11, 12, 15, 16; N5, N6).
- Failing cells (eval/baseline/probes.txt, one case):
  - probes/t3299-date-text-glued-month-3: Sheet1!B:H, J, K of rows 14 and 19-26; O:R of rows 2 and 10.

## Acceptance criteria
- [ ] Every cell of probes/t3299-date-text-glued-month-3 passes and its lines leave
      eval/baseline/probes.txt; probes/date-text-glued-month and date-text-glued-month-2 regress no cell.
- [ ] A unit test in the date-text tests covers `"1-Jul24"`, `"1/Jul24"`, `"1-Jul5"` (dates) and
      `"1.Jul24"` (`#VALUE!`), and the typed format `d-mmm-yy`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Separators and shapes the probe did not cover (dates.md UNVERIFIED list).
- Locales other than en-US.
