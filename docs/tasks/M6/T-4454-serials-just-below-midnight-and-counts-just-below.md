---
id: T-4454
title: "Serials just below midnight and counts just below an integer in EDATE, EOMONTH, WORKDAY, NETWORKDAYS, WEEKNUM, ISOWEEKNUM, DAYS, DATEDIF and YEARFRAC: probe which take YEAR's half-second carry or the 2^-22 snap, and match"
milestone: M6
lane: functions-date
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The date functions that turn a serial into a day, or a count into a whole number, do it the way
Excel does near the cut: a serial within half a second of the next midnight, and a month, day or
return-type count just below an integer. Each argument class follows the model its committed
excel-graph cells show, so the engine stops mixing three readings with no evidence for most of
them.

## Context
- Three readings are in the engine today, only two of them evidenced:
  - YEAR, MONTH, DAY and WEEKDAY (and the bound of HOUR, MINUTE, SECOND) take the half-second
    carry `floor(x + 0.5/86400)`: docs/excel-semantics/dates.md "YEAR, MONTH and DAY of a serial
    just below midnight" (probes/t3137-date-parts-near-midnight R!A1:H126, T-3137, T-3176;
    `carried_day` in crates/rustsheet-engine/src/formatter/dates.rs). That entry rules out a plain
    floor and the 2^-22 + 2^-33 snap for those functions.
  - DAYS360 rounds the serial to the nearest second before flooring (`days360_serial` in
    functions/date_and_time.rs, T-2263).
  - Every other serial argument floors plainly: EDATE and EOMONTH start (functions/date_and_time/
    month_shift.rs), WORKDAY/WORKDAY.INTL start (workday.rs), NETWORKDAYS/NETWORKDAYS.INTL start and
    end (networkdays.rs), WEEKNUM and ISOWEEKNUM serial (weeknum.rs, date_and_time.rs), DATEDIF
    (`datedif_value`), DAYS (`fn_days`) and YEARFRAC (yearfrac.rs). The only evidence is far from
    midnight: `ISOWEEKNUM(44197.99)` = 53 (probes/weeknum-modern-boundaries, dates.md line "A
    fractional serial truncates") and `DATEDIF(59.999988426,38168,"YD")` = 123 (about 1 s below 60;
    dates.md "Serials below 61 in DATEDIF, EDATE, DAYS360, WEEKNUM and YEARFRAC", gen-m6 cells),
    both of which every candidate model reproduces.
- Counts: EDATE/EOMONTH months truncate (month_shift.rs), WORKDAY days floor (workday.rs), WEEKNUM
  return_type and the WORKDAY.INTL/NETWORKDAYS.INTL weekend number truncate (`weekend_mask`;
  `WEEKNUM(45000,21.99)` = 11, dates.md). DATE and TIME snap their arguments within 2^-22 + 2^-33
  (half that for a negative value; `near_integer_whole`, dates.md "DATE: a year, month or day just
  below an integer", T-2267/T-2268). No cell probes the other counts within 2^-22 of an integer, or
  a negative month such as `EDATE(45000,-1+2^-23)` where truncation and the asymmetric snap differ.
- Sibling questions: T-4453 (text-lane counts), T-4447 (SORT), and the engineering-lane task filed
  alongside these. Reuse `carried_day`, `near_integer_count` and `near_integer_whole`; add no new
  tolerance.
- Opus: the probe has to separate four models per argument (plain floor, half-second carry,
  rounding to the second as DAYS360 does, 2^-22 snap), at bases where the carried day changes the
  answer (a month end for EDATE/EOMONTH, a Friday for WORKDAY, a week boundary for WEEKNUM, a
  year end for YEARFRAC basis 1, serial 59/60/61 for the fictitious 1900-02-29), and DAYS360's own
  rounding is inferred from one engine comment rather than a cited cell.

## Acceptance criteria
- [ ] A committed probe case (excel-probe skill, excel-graph sidecar) gives, for each serial
      argument named above (EDATE, EOMONTH, WORKDAY, WORKDAY.INTL, NETWORKDAYS, NETWORKDAYS.INTL
      start and end, WEEKNUM, ISOWEEKNUM, DAYS, DATEDIF, YEARFRAC, DAYS360), the serials
      `fl(N-0.5/86400)`, one ulp below it, `N-0.4/86400`, `N-0.6/86400` and `N-2^-22-2^-33`, built
      exactly in formulas as t3137 did, at a base N where the day matters.
- [ ] The same case brackets the counts (EDATE/EOMONTH months, WORKDAY/WORKDAY.INTL days, WEEKNUM
      return_type, the .INTL weekend number) at `n-2^-22-2^-33`, `n-2^-22-9*2^-36` and, for the
      signed counts, `-n+2^-23` and `-n+2^-22`.
- [ ] docs/excel-semantics/dates.md gets an entry (or extends "YEAR, MONTH and DAY of a serial just
      below midnight") naming the model of every probed argument, citing the cells; DAYS360's
      rounding cites a cell.
- [ ] The engine reads each argument with its model through the existing helpers; every probe cell
      passes bit-exact, and the t3137 cells keep passing.
- [ ] No new known-failure line in eval/baseline/; `make check` green; reviewers approve.

## Out of scope
- DATEVALUE/TIMEVALUE text parsing, TEXT and cell display of such serials (functions-text, M10).
- The 1904 date system (a follow-up if the 1900 probe shows a carry the 1904 code paths must copy).
- COUP*, ACCRINT and other financial date arguments (functions-financial).
