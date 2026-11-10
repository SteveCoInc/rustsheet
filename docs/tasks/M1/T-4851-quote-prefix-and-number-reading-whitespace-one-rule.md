---
id: T-4851
title: "Quote prefix and number-reading whitespace: one rule for 'this text reads as a number'"
milestone: M1
lane: engine
status: done
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
- [x] `value_needs_quoting` uses the typed-input reader (now `Model::text_needs_quote_prefix`).
- [x] Probe tab/NBSP/newline-padded numbers under `+0`, VALUE and COUNTIF; the trims follow the evidence and the quirk is recorded in docs/excel-semantics/coercion.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other duplicate concepts of the audit (T-4847 to T-4855 each own one).

## Outcome
- `update_cell_with_text` quotes through `Model::text_needs_quote_prefix` (cast.rs). It reads number text with typed
  input's own reader, `parse_typed_number`, and the same currencies (`typed_currencies`, now shared with
  `reenter_printed_input`), so `50%`, `$5`, `1/2/2020` and `10:30` keep the quote prefix and stay text when typed
  back in. utils.rs `value_needs_quoting` and its test moved there (test_quote_prefix.rs).
- Probe probes/t4851-number-text-whitespace (100 cells, excel-graph): `+0`, VALUE and a COUNTIF criterion trim only
  the space. Tab, LF, CR, VT, FF, NBSP, U+2003, U+202F and U+3000 pads are `#VALUE!` or no match. `+0` trimmed all
  Unicode whitespace (`str::trim` in `cast_number_in`) and now trims the space only. VALUE and criteria already did.
  coercion.md "Number text trims only the space (T-4851)" records it, with the `COUNTIF(..,"> 0")` = 0 quirk.
- Eval delta: the new case is 100/100 (main: 73/100, the 27 non-space `+0` cells off); see the PR for the full
  eval check.

