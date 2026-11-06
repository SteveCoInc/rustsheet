---
id: T-4507
title: "Land the locale, calendar and fraction probes of the T-3666 generalization with their semantics notes"
milestone: M10
lane: functions-text
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Keep the Excel evidence of an unmerged generalization branch (locales, calendars, fraction placeholders, commas, AM/PM
hours) on main, with its rules in the semantics notes, so later work starts from it.

## Context
Branch `ccr-c02f0513-m4y8cp` (2026-11-03..05) generalized T-3089, T-3598, T-3666, T-3710 and T-3711 while other runs
landed narrower fixes for the same tasks on main; a merge conflicted in 23 files, so the branch is salvaged in two
steps: this one (probes and notes, no engine change) and T-4508 (the generated locale tables). Its probes name the
branch's own task ids T-4042 (locale names), T-4043 (fraction placeholders) and T-4061 (calendars), which never
reached main; their work is T-4508 and T-4515.

## Acceptance criteria
- [x] The branch's 27 probe cases and Excel sidecars are on main (`t3710-ampm-hour-tokens` renamed
  `t3710-ampm-hour-layouts`, main having a case of that name); the cells main fails are known failures.
- [x] number-formats.md records each rule with its evidence and what ours does.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes (T-4508 locale tables, T-4515 fraction and comma model).

## Outcome
Done. 27 cases, 11460 Excel cells (excel-graph, 2026-11-04): main passes 3308 (2989 before T-4480 landed); the rest are known
failures, owned by T-4508 (locale and calendar names, calendar digits 0x15-0x17 and the top bit: t3666-*, t4061-*) and
T-4515 (t4043-*, t3711-fixed-denominator-then-placeholder, and t3598-decimal-part-commas rows 23, 24, 26,
t3666-review-followups A7:A12). number-formats.md: new "Which LCIDs have a locale" and "Calendar names per locale",
the calendar-digit rows 15-17 and 80+, "Placeholders after a fraction", and more evidence in "Commas after the decimal
point", "Hours with AM/PM" and the `B2` token codes.
