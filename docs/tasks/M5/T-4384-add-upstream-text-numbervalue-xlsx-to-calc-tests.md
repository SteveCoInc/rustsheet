---
id: T-4384
title: "Add upstream TEXT/NUMBERVALUE.xlsx to calc_tests"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The Excel-authored workbook upstream IronCalc added on 2026-11-05, `TEXT/NUMBERVALUE.xlsx` (ironcalc `ae1fccd3`),
is in `crates/rustsheet-xlsx/tests/calc_tests` with every cell passing.

## Context
Upstream triage, steward 2026-11-05 (second run). The NUMBERVALUE code fixes in `f6b52dee` and `ae1fccd3` need no
port: our `numbervalue_of` in `functions/text/string_format.rs` already follows oracle evidence for the same
classes, and is ahead of the upstream rules in places (docs/excel-semantics/text.md, "NUMBERVALUE removes group
separators only before the decimal separator", T-3647, T-3746, T-2441, T-3742): whitespace inside the text,
NBSP and U+3000 inside the digits, separators read by their first character, an empty group separator, and
text of `%` signs alone being 0.

The workbook may still add Excel-computed cells we do not have. Recover it with
`git -C target/upstream-ironcalc show ae1fccd3:xlsx/tests/calc_tests/TEXT/NUMBERVALUE.xlsx`
(`scripts/upstream-log.sh` keeps the clone). First check it is Excel-authored (the suite sets
`require_excel_authored`; `rustsheet-eval snapshot` shows the application), and leave it out with a note here if it
is not. `build.rs` makes a strict test per workbook, so add it only when all its cells pass (precedent T-3857,
T-1517, T-0965). Check cells against the workbook's own cached values, never against the upstream commit messages
(untrusted). When a cell fails, fix it here if S-sized, or else file a follow-up naming the cells and leave the
workbook out. Record any new quirk in `docs/excel-semantics/text.md`.

## Acceptance criteria
- [ ] The workbook passes in full and is in calc_tests (`rustsheet-eval show` on it shows every cell passing;
      baseline updated, 0 regressions), or it is left out with a filed follow-up task naming its failing cells (or
      with a note that it is not Excel-authored).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Porting upstream's code changes (ours already follows Excel's evidence for these classes).
