---
id: T-3885
title: "TEXTJOIN: a text ignore_empty is #VALUE! even when the delimiter is an error"
milestone: M5
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
TEXTJOIN returns #VALUE! for a text ignore_empty even when the delimiter is an error value; an error delimiter with a numeric, boolean, blank or error ignore_empty keeps its own error.

## Context
Found by the T-3512 arg-pairs batch (`make eval-show CASE=gen-m5/d0c5e5dff134d787.case.json`). `TEXTJOIN(<#DIV/0! ref>,<text ignore_empty>,<values>)`: Excel `#VALUE!`, ours `#DIV/0!`. Do not change the passing cells: A286, A292, A298, A304, A328, A334 (number, boolean, blank or error ignore_empty) stay `#DIV/0!`; A349 and A352 (text ignore_empty, blank delimiter) are `#VALUE!` and pass. Rule recorded in docs/excel-semantics/text.md.
- d0c5e5dff134d787: Calc!A310, A316, A322.

## Acceptance criteria
- [x] d0c5e5dff134d787 Calc!A310, A316 and A322 leave eval/baseline/gen-m5.txt; no new known failure.
- [x] The probed rule is in docs/excel-semantics/text.md.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Every cell this task owns has the right type, error and text (decision 0083).

## Out of scope
- Other TEXTJOIN argument kinds (T-2863).

## Handoff (resumed)
Tried: in `fn_textjoin` (functions/text/concatenate.rs), when the delimiter is an error and the
`ignore_empty` value is `CalcResult::String`, return `#VALUE!`; other kinds keep the delimiter's
error. Unit test `error_delimiter_with_text_ignore_empty` passes and A310/A316/A322 of
gen-m5/d0c5e5dff134d787 are fixed.

Failed: `make eval-baseline` refuses a visible-parity REGRESSION: gen-m5/1d22a002d4f380b3 Calc!A274
`=TEXTJOIN(VLOOKUP(OR(Data!E7:E9),Data!B3:E12,3), IF(IFNA(1.005,""),LOWER(Data!D12)), ...)` is
Excel `#N/A`, ours now `#VALUE!`. There the delimiter is a #N/A *function result* and the text
ignore_empty a *computed* value (LOWER); in the d0c5 cells the delimiter is a #DIV/0! *cell
reference* (Data!G9) and ignore_empty a *cell reference*. The task's rule is therefore too broad;
which variable decides (delimiter error kind, delimiter reference vs computed, ignore_empty
reference vs computed) is unprobed.

Next: probe with `excel-probe` (excel-graph is OK) the 2x2x2 of delimiter {ref, computed} x error
{#DIV/0!, #N/A} x ignore_empty {ref, computed text}, narrow the code condition to the evidenced rule,
fix the text.md entry (line ~1383) and rerun `make eval-baseline`.

## Outcome
Probed which variable decides (probes/t3885-textjoin-error-delimiter-text-ignore-empty, 136 cells,
excel-graph): it is whether the delimiter is a *reference*. With a reference to an error cell (a cell,
a range, IF or INDEX returning one) Excel reads ignore_empty first: text is `#VALUE!` and an error is
its own error (`<#DIV/0! cell>,<#N/A cell>` is `#N/A`); with an error *value* (literal, `1/0`, `NA()`,
VLOOKUP) the delimiter's error always wins, which is why gen-m5/1d22a002d4f380b3 A274 stays `#N/A`.
Error kind and whether ignore_empty is a reference or computed do not matter.
`Model::textjoin_delimiters` now evaluates the delimiter once in reference context
(`evaluate_reference_arg`, as the texts are read), so it knows both the value and whether it was a reference,
and `fn_textjoin` lets ignore_empty's cast error win only for a reference. Review round 1 (excel-fidelity) found
that round 1's second evaluation on the error path doubled the work at each nested TEXTJOIN delimiter (2^depth;
`#VALUE!` from the evaluation budget at depth 24). Test `nested_error_delimiters_are_evaluated_once` (depth 30) covers it.
Follow-up: T-3909 (lifted ignore_empty with a reference error delimiter, unprobed). Rule in
docs/excel-semantics/text.md "TEXTJOIN: ignore_empty is read before a reference delimiter's error".
Eval delta: gen-m5/d0c5e5dff134d787 Calc!A310, A316, A322 leave the baseline (-3 known failures);
the new probe passes 136/136; no regression.
