---
id: T-3869
title: "xlsx import: refuse a workbook whose formula passes Excel's 256-parenthesis or sign cap, as Excel does"
milestone: M2
lane: engine
status: done
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
- [x] A workbook with a 257-parenthesis or a past-the-sign-cap cell formula is refused with `XlsxError::Workbook`, and
      256 parentheses or 1,024 signs before `1` still load (tests in rustsheet-xlsx).
- [x] syntax.md records the importer's refusal.
- [x] `make check` green; reviewers approve.

## Out of scope
- Defined-name bodies and conditional-format and data-validation formulas (unprobed for these caps;
  T-3872 probes them).

## Outcome
- The importer refuses a workbook whose cell formula parses to a 257-parenthesis or past-the-sign-cap
  parse error, with `XlsxError::Workbook("Formula in Sheet1!A100: <parser message>")`
  (`call_cap::refuse_past_syntax_cap`, read from the new `Node::syntax_cap_error`, which shares its
  walk with `Node::has_parse_error`). Before, such a workbook loaded with `#ERROR!` in the cell.
- Decided: cells only. Name bodies and rule formulas past these caps still load as a parse error;
  Excel's handling there is unprobed, so T-3872 (research, sonnet) probes it.
- Test: `formula_past_the_parenthesis_or_sign_cap_refuses_the_workbook` in
  `rustsheet-xlsx/tests/malformed_import.rs` (256 / 257 parentheses, 256 / 257 levels of `-(`, 1,024 /
  1,025 minus and plus signs before `1`, 1,022 / 1,023 before `(1)`, 200 levels of `-(` around 824 /
  825 signs), run on the parser tests' small stack.
- syntax.md "Formula nesting depth cap" records the refusal.
- Eval delta: none (the refused stored probes were dropped after their runs; no case changes).
