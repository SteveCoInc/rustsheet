---
id: T-4397
title: "WORKDAY and NETWORKDAYS holidays given as text (literal, cell, array; number, date and currency text) and DAYS with two error arguments: probe the rules ours guesses"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
WORKDAY, WORKDAY.INTL, NETWORKDAYS and NETWORKDAYS.INTL read a holiday given as text the way Excel does in the forms no
Excel cell covers yet (a text literal, a cell holding date or currency text, an array of such text), and DAYS with an
error in both arguments returns Excel's error. Each rule is backed by excel-graph cells and recorded in dates.md.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 8465f6f1, excel-graph sidecars): every failing cell of the date
  functions passes or has an open owner, so this triage read the rules dates.md still marks UNVERIFIED or unprobed.
  No open task covers these (T-2591 is the weekend argument, T-1954 the lift controls, T-2864 the empty-text and error
  kinds of 20 date parameters and not the holidays arrays).
- dates.md "WORKDAY and NETWORKDAYS holidays: blank and number-text cells, and the error order (T-2270, T-2272)", its
  "Ours" paragraph: "Also UNVERIFIED: the scalar branch accepts any text `cast_number` reads (a literal `"45001"`, a
  formula's number text, date, currency or percent text) as a holiday; the cells only show holidays reached through a
  reference." Probed so far: number text in a cell (`WORKDAY(18260,-30,Data!C4)` with C4 `"1"`, gen-m6/1aa50086e42bff9a
  A67) and in an array (`{"1.5"}`, probes/t2360-array-lambda-holidays W1-W15). Not probed: date text (`"3/16/2023"`),
  currency or percent text, and any text given as a literal.
- dates.md "NETWORKDAYS.INTL: a scalar holiday ignores the weekend argument" (T-3537): a scalar holiday is dropped only
  on a Saturday or Sunday whatever the weekend, while a reference, range or array holiday follows the weekend. Whether a
  text literal holiday is "scalar" there is unprobed.
- dates.md "DAYS error order" (T-3536): "Two error arguments at once (which one wins) is not probed; the code returns the
  leftmost."
- Ours today (scratch case on a release build of 8465f6f1; A1 `"45001"`, A2 45002, A3 `"3/20/2023"`; 45000 is
  Wednesday 2023-03-15):
  - `NETWORKDAYS(45000,45030)` 23; with holiday `"45001"`, `"3/16/2023"`, `"$45,001"`, `{"45001"}`, A1 or A3: 22 each;
    with A1:A2: 21.
  - `WORKDAY(45000,5,"45001")` 45008; `WORKDAY(45000,5,A1:A3)` 45012.
  - `DAYS(NA(),1/0)` `#N/A`, `DAYS(1/0,NA())` `#DIV/0!` (leftmost).
- Code: crates/rustsheet-engine/src/functions/date_and_time/ (`Model::workday_of`, `Model::networkdays_of`,
  `holiday_serials`, `collect_serial_numbers`, `Holidays::scalar`); `fn_days`.

Probe (excel-probe skill, excel-graph, tag M6), probes/t4397-holiday-text-forms and probes/t4397-days-two-errors:
- For each of WORKDAY, WORKDAY.INTL (weekend 1 and 13), NETWORKDAYS and NETWORKDAYS.INTL (weekend 1 and 13), with a
  weekday holiday and a Saturday holiday inside the span: the holiday as a number literal (control), as a text literal
  (`"45001"`), as date text (`"3/16/2023"`, `"March 16, 2023"`), as currency and percent text (`"$45,001"`,
  `"4500100%"`), as a cell holding each of those texts, as a one-element array of each (`{"3/16/2023"}`), and inside a
  two-cell range next to a number.
- Wrap each in `ERROR.TYPE(..)` too where the bare result may be an error.
- `DAYS(NA(),1/0)`, `DAYS(1/0,NA())`, `DAYS(#REF!,#NAME?)`, the same pairs as one-cell references to error cells, and
  `DAYS(NA(),"abc")` / `DAYS("abc",NA())` as controls.

## Acceptance criteria
- [x] The two probe cases are committed with excel-graph oracle sidecars.
- [x] Every probe cell passes: the holiday text forms through the functions' holiday reader, the scalar/reference split
      of NETWORKDAYS.INTL included, and DAYS's choice between two errors in `fn_days`. Unit tests pin one text-literal
      holiday and one two-error DAYS call.
- [x] dates.md's two UNVERIFIED statements above are replaced by the probed rules, citing the cells.
- [x] `make eval-baseline` records no regression; no cell of probes/networkdays-intl-holiday-* or
      probes/t2360-array-lambda-holidays changes.
- [x] `make check` green; reviewers approve.

## Out of scope
- The weekend argument (T-2591) and the lift controls (T-1954).
- Empty text and errors as date arguments (T-2864).

## Outcome
- Two probe cases, filled by excel-graph: probes/t4397-holiday-text-forms (516 cells: WORKDAY, WORKDAY.INTL weekend 1
  and 13, NETWORKDAYS, NETWORKDAYS.INTL weekend 1 and 13, with a Thursday or Saturday holiday as a number, text literal,
  cell, one-element array or two-cell range, for number, m/d/yyyy, long-date, currency and percent text, each also under
  ERROR.TYPE) and probes/t4397-days-two-errors (36 cells). Generator: docs/research/t4397/gen_probe.py.
- Excel: every text form is a holiday, read as the serial its text gives; a text literal is a scalar holiday for
  NETWORKDAYS.INTL's weekend rule (a Saturday is dropped, 10), while a cell or array holding the text follows the weekend
  (9). DAYS with two errors returns the leftmost, and an error beats unreadable text in either position.
- Ours already matched every cell (552/552), so there is no engine change. Two unit tests pin a text-literal holiday and
  DAYS's two-error order. dates.md: the two UNVERIFIED statements are replaced by the probed rules.
- Eval delta: +552 passing probe cells (new coverage, probes has no baseline); no other cell changes; no regression.
