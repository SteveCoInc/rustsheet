---
id: T-3901
title: "Probe typed mixed-separator and clock-first dates in a day-first locale"
milestone: M14
lane: functions-date
status: blocked
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Typed mixed-separator dates (`2025/07-01`, `1-Jul/2025`) and clock-first date-times in a day-first locale take Excel's value and number format.

## Context
T-3796 and T-3852 evidenced en-US only (probes/date-text-mixed-separator-typed, date-text-typed-clock-first-shapes). `mixed` in `date_text.rs` skips day-first locales and keeps the joined separators; `leading_clock_shape` reads the date in the locale's order. Needs an excel-graph locale probe (or a desktop question for M14).

## Acceptance criteria
- [ ] A probe case records Excel's typed value and format in a day-first locale; the engine matches or the finding is recorded in docs/excel-semantics/dates.md.
- [ ] `make check` green; reviewers approve.

## Blocked because
Needs the `excel-desktop` backend (Windows with Excel), like T-3865. excel-graph cannot answer: it calculates in the
SharePoint site's regional settings, and its canary pins those to en-US (`DATEVALUE("1/2/2001")-DATEVALUE("1/1/2001")`
must be 1; docs/architecture/oracle.md, "Locale is a site setting"), so every fill in a day-first locale fails by
design, and a case has no per-workbook locale field. On 2026-11-03 `rustsheet-oracle status` showed `excel-desktop n/a`.
Moved from M6 to M14 (decision 0039: desktop-only questions live in Phase 4). No probe case is committed: a pending
case would send the scheduled oracle workflow to excel-graph, which would answer in en-US.
When the desktop backend is up, type in an en-GB (d/m/yyyy) Excel, reading value, ISNUMBER and `CELL("format")`:
`1/7-2025`, `1-7/2025`, `2025/07-01`, `2025-07/01`, `1-Jul/2025`, `1/Jul-2025`, `1/7-25`, `10:00 1/7/2025`,
`10 AM 1/7/2025`, `10:00 1-Jul-2025`, `10:00 1/7`, and `13/1-2025` (a day over 12 in first place). The engine today
gives a day-first mixed numeric date the locale's order with `/` as its format (`1/7-2025` is `d/m/yyyy`), not
Excel's en-US `m/d/yyyy` (`date_text.rs` `parse_separated`, `TODO(T-3901)`); that is unevidenced either way.

## Out of scope
-
