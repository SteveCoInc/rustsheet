---
id: T-3944
title: "DATEVALUE of a bare clock: 0 in a 1904 workbook (DATEVALUE(\"10 A.\")), probe both systems"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
DATEVALUE of clock-only text (`"10 A."`, `"10:00"`, `"10 AM"`) matches Excel in both date systems.

## Context
In a 1904 workbook `DATEVALUE("10 A.")` is 0 on excel-graph (probes/t3920-typed-a-period-1904 G6, 2026-11-03);
the engine gives #VALUE! (`Model::parse_datevalue_text` in `functions/date_and_time.rs` reads no date in a bare clock).
In the 1900 system `DATEVALUE("10 A.  ")` is 0 too (probes/t3948-letter-period-gate-edges G14, excel-graph 2026-11-03).
Whether the 1900 system also gives 0 (or #VALUE!), and whether other clock-only texts (`10:00`, `10 AM`, `25:00`)
behave the same, is unprobed: write a probe case with DATEVALUE of a handful of clock-only texts in a 1900 and a
1904 workbook, fill it, then make the engine match and record the rule in docs/excel-semantics/dates.md.

## Acceptance criteria
- [ ] A probe case records DATEVALUE of clock-only text in both date systems; probes/t3920-typed-a-period-1904 G6 and t3948-letter-period-gate-edges G14 pass and the engine matches the new case.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
