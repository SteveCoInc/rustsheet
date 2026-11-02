---
id: T-3869
title: "xlsx import: refuse a workbook whose formula passes Excel's 256-parenthesis or sign cap, as Excel does"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The xlsx importer refuses a workbook whose cell formula passes Excel's 256-parenthesis cap (`MAX_PARENTHESES`) or its
sign cap (`MAX_SIGNS`), as it already refuses one past the 65-call cap. Excel for the web refuses such a workbook whole
(createSession 501 `FileCorruptTryRepair`).

## Context
- Follow-up of the T-3689 review (design reviewer, MINOR). Since T-3338 (signs) and T-3689 (parentheses), a stored formula
  past either cap loads as a parse error (`#ERROR!`), not as a refused workbook. syntax.md "Formula nesting depth cap"
  records Excel's refusal of the stored forms: probes/t3172-* and t3266-* refused at createSession, and those cases were
  dropped after the run.
- The call cap's importer check is `crates/rustsheet-xlsx/src/import/call_cap.rs` (`XlsxError::Workbook`). The parser
  messages are `TOO_MANY_PARENTHESES_MESSAGE` and `TOO_MANY_SIGNS_MESSAGE` (depth.rs).
- Decide whether defined-name bodies and conditional-format and data-validation formulas get the same check, as T-3143
  and T-3263 did for calls (unprobed for these caps).

## Acceptance criteria
- [ ] A workbook with a 257-parenthesis or a past-the-sign-cap cell formula is refused with `XlsxError::Workbook`, and
      256 parentheses or 1,024 signs before `1` still load (tests in rustsheet-xlsx).
- [ ] syntax.md records the importer's refusal.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
