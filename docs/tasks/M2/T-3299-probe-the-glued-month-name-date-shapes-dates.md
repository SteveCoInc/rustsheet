---
id: T-3299
title: "Probe the glued month-name date shapes dates.md leaves UNVERIFIED after T-1906 (1Jul5, 1/Jul2026, 1-Jul26, July1 , 2026 5) and the typed format of 1-Jul2026"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence (excel-graph) decides each glued month-name date shape that dates.md "Date text with a month name
glued to digits (no separator)" still lists as UNVERIFIED. The section then states Excel's rule for the shape
family, and every engine cell that disagrees has an owning task.

## Context
- Gap triage 2026-10-30e. Every visibly-off M0-M2 cell has an open owner and gen-m2 is at 100%, so the open M2 risk
  is in rules the engine extends by inference. Date text feeds YEAR, MONTH and DAY (M2 functions), VALUE, `0+`
  coercion and criteria, all through `split_glued_month` (crates/rustsheet-engine/src/formatter/date_text.rs, used
  by `parse_date`).
- What is verified: probes/date-text-glued-month (rows 1-24) and probes/date-text-glued-month-2 (T-1879, 106 cells).
  T-1906 implemented them.
- UNVERIFIED, from the section's list (the engine's answer comes from the rule it generalised, not from Excel):
  - `"1Jul5"`: a single glued digit after a day, read by the engine as year 05, like `"1July1"` = 2001-07-01;
  - other separators between day and glued month-year: `"1/Jul2026"`, `"1.Jul2026"` and `"1,Jul2026"`
    (a space, `"1 JUN2021"`, and a hyphen, `"1-Jul2026"`, are verified);
  - `"1-Jul26"` (two-digit glued year after a hyphen) next to the verified `"1-Jul2026"`;
  - the hand-derived `#VALUE!` of `"July1 , 2026 5"`, `"1Jul1 , 2026"` and `"July1,, 2026"`;
  - the number format typed `"1-Jul2026"` gets: the oracle's ENTERED log records it, and the other typed glued forms
    get `d-mmm`, `mmm-yy` or `d-mmm-yy`.
  The clock-year results are skipped: year-less forms depend on the oracle's clock, so score them through MONTH and
  DAY only, as date-text-glued-month-2 does. Non-en-US month names are skipped too.
- Reuse date-text-glued-month-2's layout: columns for DATEVALUE's MONTH/DAY/YEAR, VALUE and `0+` minus DATEVALUE,
  a COUNTIF criterion column, and typed-input cells with the ENTERED format log. Error cells hold ERROR.TYPE - 100 so
  the error kind is scored as a number.
- Add a few neighbours around each shape (a two-digit and a four-digit year, an abbreviated and a full month name,
  upper and lower case). Then the section can state a rule rather than a list.

## Acceptance criteria
- [x] A new probes case `probes/t3299-date-text-glued-month-3` (milestone M2) with an excel-graph oracle sidecar. It
      covers every shape above with the neighbours, through DATEVALUE, VALUE, MONTH/DAY, a criterion and typed input.
      At least 80 scored cells.
- [x] dates.md's section states Excel's result for each shape with cell citations and drops the UNVERIFIED items the
      probe settles; the typed format of `"1-Jul2026"` is recorded.
- [x] Every cell the engine fails is a known failure in eval/baseline/probes.txt (`make eval-baseline`,
      0 regressions). Each is owned by a task filed in this PR (`functions-date` or `engine` for
      `split_glued_month`) or by an existing open one, and `make exit-owners MILESTONE=M2` reports 0.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing the engine (the owning tasks).
- Clock-year values for clocks other than the oracle's, and other locales.
- Non-glued date text (dates.md's other sections).

## Outcome
- New probe `probes/t3299-date-text-glued-month-3` (excel-graph, 2026-11-03): 35 text shapes through
  DATEVALUE (MONTH/DAY/YEAR), VALUE and `0+` minus DATEVALUE, MONTH/DAY of the text, COUNTIF and SUMIF
  criteria, plus 10 typed-input cells. 363 scored cells. Every year is 2024, so no scored cell depends
  on the oracle's clock. A first fill used 2026, the clock's year, which made the year and criteria
  cells ambiguous, so the case was moved and refilled.
- dates.md records Excel's rule for each shape and drops the settled UNVERIFIED items:
  - `"1Jul5"` is 2005-07-01, confirming T-1906's inference.
  - A slash or space before a glued month-year is a date; a period or comma is `#VALUE!`.
  - A hyphen or slash takes a one- or two-digit glued year (`"1-Jul24"`, `"1/Jul24"`, `"1-Jul5"`).
  - The hand-derived `#VALUE!` shapes are `#VALUE!`.
  - Typed `"1-Jul2024"`, `"1-Jul24"`, `"1/Jul2024"`, `"1/Jul24"`, `"1-JULY2024"` and `"1Jul5"` are dates
    with format `d-mmm-yy`.
  - The raw ENTERED log is in docs/research/t3299-glued-month-typed-input-2026-11-03.md.
- The engine fails 89 cells: the slash or hyphen shapes with a one- or two-digit glued year (rows 14,
  19-26; typed N2, N10). They are known failures owned by the new T-3919 (M2, functions-date, sonnet).
- Also filed T-3922 (M2, research, sonnet): the three MINOR coercion.md wording findings from the
  fidelity review of T-3559 (#2963).
- Eval delta: probes +1 case, 274/363 cells pass, 89 new known failures, 0 regressions.
