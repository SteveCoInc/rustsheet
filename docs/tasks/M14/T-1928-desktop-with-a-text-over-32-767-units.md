---
id: T-1928
title: "Desktop: & with a text over 32,767 units as the left operand"
milestone: M14
lane: functions-text
status: dropped
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Know what `&` does when its left operand is a text over 32,767 UTF-16 units (desktop Excel), and match it.

## Context
T-1913 found excel-graph refuses the workbook for `REGEXREPLACE(REPT("a",32767),"a","bb")&"c"`, `&""`, `&R`, LET and IF shapes, so only the right-operand case is measured (text.md "SUBSTITUTE result longer than 32,767 characters"). Ours truncates to 32,767 like the right-operand case. Needs desktop Excel (decision 0039, M14).

## Acceptance criteria
- [ ] A probe with the long text as the left operand has desktop oracle values; ours matches or the gap is baselined; text.md updated.
- [ ] `make check` green; reviewers approve.

## Blocked because
Phase 4 (decision 0039): not a "Needs owner" item, and it blocks no milestone or phase exit. The evidence needs desktop Excel. Unblock when `cd tools && uv run rustsheet-oracle status` shows `excel-desktop OK` (2026-10-30: `excel-desktop  n/a`).

## Out of scope
-

## Outcome (dropped, 2026-11-04, decision 0266)
Dropped: desktop-Excel work is out of scope (owner, 2026-11-04). It needed the `excel-desktop` oracle backend, which
was never set up. The context above is kept in case a later decision reopens desktop work.
