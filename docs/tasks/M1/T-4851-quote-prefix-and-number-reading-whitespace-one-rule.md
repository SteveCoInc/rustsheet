---
id: T-4851
title: "Quote prefix and number-reading whitespace: one rule for 'this text reads as a number'"
milestone: M1
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The engine has one answer to "does this text read as a number" for the quote prefix, coercion, VALUE and criteria, each variant justified by evidence.

## Context
`value_needs_quoting` (utils.rs), used by `update_cell_with_text` (model.rs), only checks `parse_excel_number` and bool parsing, while typed input (`set_user_input`) uses `parse_typed_number` (percent, currency, dates, times): `update_cell_with_text("50%")` or `("1/2/2020")` gets no quote prefix and becomes a number when re-entered. Whitespace: `cast_number_in` (cast.rs) trims all Unicode whitespace; VALUE (`value_number_text_with`, cast.rs) and criteria (`parse_criterion_number`) trim only spaces (criteria rejects other whitespace). `"1"&CHAR(9)` may coerce under `+0` but not VALUE — unprobed (text.md documents only VALUE's inner tab).

Found by the T-4856 duplicate-concept audit (one definition per concept, CLAUDE.md directive 3). Excel decides each rule (directive 1): where the existing variants disagree and no probe in `eval/` or `docs/excel-semantics/` settles which one Excel follows, probe it first (`excel-probe` skill) and record the finding.

## Acceptance criteria
- [ ] `value_needs_quoting` uses the typed-input reader.
- [ ] Probe tab/NBSP/newline-padded numbers under `+0`, VALUE and COUNTIF; the trims follow the evidence and the quirk is recorded in docs/excel-semantics/coercion.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).
