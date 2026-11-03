---
id: T-3920
title: "Probe the unprobed edges of the typed 'N A. F/D/Y' reading (AM., H:M hours from 24, 1904 day 0, formula readers of text-staying shapes)"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: opus
depends_on: [T-3907]
---

## Goal
The engine reads the unprobed edges of Excel's typed `N A. F/D/Y` reading (T-3907) as Excel does, or dates.md says why not.

## Context
T-3907 decoded `10 A. 7/1/2025` (2^32 - 22 s + clock + fraction + date; `date_clock::letter_period_parts`) on
probes/t3907-typed-clock-a-period-shapes and -edges. Reviewers of that PR (MINOR findings) noted what stays unprobed,
listed in docs/excel-semantics/dates.md "Typed clock-first date-times": (1) `AM.` (the engine reads it as `A.` by
symmetry); (2) an `H:M` clock with an hour of 24 or more (`25:00 A. 7/1/2025`; the engine leaves it text); (3) a 1904
workbook, where the no-date form `10 A. 5` has day 0 before the epoch (the engine leaves it text); (4) the formula
readers (VALUE, `0+`, DATEVALUE, TIMEVALUE) of the shapes that stay text when typed (`10 A. 7-2025`, `10 A. 7/1`,
`60 A. 7/1/2025`): the engine reads some through the older `10 A.` + date path. Also: shapes row 11 (`10 A. 7/1/25`,
C11 `MONTH*100+DAY+MOD`) reads 25 January of the current year, so a refill in a year whose offset crosses a leap day
would disagree with the engine on the pinned 2022 clock; make that row year-proof (MONTH only, or a description note).
And (design MINOR) `parse_formatted::is_am_pm_word` could call `date_clock::am_pm_word(word).is_some()`.

## Acceptance criteria
- [ ] A probe case records Excel's values for (1)-(4); the engine matches each cell or dates.md records the gap with an owner task.
- [ ] Shapes row 11 no longer depends on the refill year; `is_am_pm_word` reuses `am_pm_word`.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
