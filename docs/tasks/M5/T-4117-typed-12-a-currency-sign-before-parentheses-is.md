---
id: T-4117
title: "Typed $(12): a currency sign before parentheses is a negative number in Excel ($(١٢) is -12), text in ours"
milestone: M5
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Text `$(12)` (a currency sign, then a parenthesised amount) typed into a cell, and read by coercion if Excel does,
becomes the number Excel makes of it.

## Context
probes/t3891-typed-script-digit-shapes B29:C29 (excel-graph 2026-11-04): typed `$(١٢)` is -12 with the long
currency format `[$$-ar-SA,200] #,##0.00_);[Red]([$$-ar-SA,200]#,##0.00)`; the engine keeps it text because
the ASCII reading `$(12)` fails `parse_formatted_number` (formatter/parse_formatted.rs; T-3891 maps the
script digits and reuses the ASCII parser). The ASCII form is unprobed: probe first (excel-probe) typed
`$(12)`, `$(1,234.5)`, `$ (12)`, `($12)`, `(12$)`, and `"$(12)"+0`, VALUE and NUMBERVALUE of each, with the
ENTERED formats.

## Acceptance criteria
- [ ] probes/t3891-typed-script-digit-shapes B29:C29 and the new probe's cells pass; the rule is in
      docs/excel-semantics/coercion.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Script-digit tags (T-3891).

## Outcome
`parse_percent_or_currency` reads a currency sign, optional spaces, then a parenthesised unsigned amount as a negative number
(`$(12)`, `$ (12)`, `$(1,234.5)`, `$(0)`), typed and coerced; `$(-12)`, `-$(12)`, `$((12))`, `$(12`, `(12$)` and `$(12)%` stay text.
Probe probes/t4117-typed-currency-paren (excel-graph 2026-11-04) passes 75/75; t3891-typed-script-digit-shapes B29:C29 now pass
(baseline shrank by 2 cells, 1 new passing case). Rule in docs/excel-semantics/coercion.md. The ENTERED format log is docs/research/t4117-typed-currency-paren-2026-11-04.md; signs other than `$` and `€` are unprobed.

## Outcome (opus resume of the parallel handoff, PR #3200)
A second attempt (agent/engine/T-4117-currency-paren, handed off to opus) ran in parallel with the one above. After that
one merged, this PR keeps only what it adds:
- Probe probes/t4117-currency-parens (A1:A34, excel-graph), 170/170 cells pass. It adds `€`, `£`, E-notation, `%`,
  mixed fractions, signed, empty and unclosed amounts inside the parentheses. ENTERED log in
  docs/research/t4117-currency-parens-2026-11-04.md.
- A mixed fraction in parentheses is negative: `$(1 1/2)` is -1.5 in `# ?/?` and `$(3 1/12)` is in `# ??/??`
  (`read_mixed_fraction`, `mixed_fraction_denominator_is_wide`). Typed `$(1 1/2%)` is text in `0.00%`
  (`is_negative_currency_percent_fraction`).
- The rows are folded into coercion.md "Currency sign before parentheses is negative".
Eval: the new case passes, with 0 regressions.
