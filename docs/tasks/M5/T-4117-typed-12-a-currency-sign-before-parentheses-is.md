---
id: T-4117
title: "Typed $(12): a currency sign before parentheses is a negative number in Excel ($(١٢) is -12), text in ours"
milestone: M5
lane: engine
status: done
priority: P2
size: S
tier: opus
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
- [x] probes/t3891-typed-script-digit-shapes B29:C29 and the new probe's cells pass; the rule is in
      docs/excel-semantics/coercion.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Script-digit tags (T-3891).

## Outcome
Probed ASCII forms (probes/t4117-currency-parens A1:A34, excel-graph; ENTERED log in
docs/research/t4117-currency-parens-2026-11-04.md). `$(12)`, `$ (12)`, `$( 12 )`, `$(1,234.5)`,
`$(1e3)` (`0.00E+00`) and the mixed fractions `$(1 1/2)` and `$(3 1/12)` (`# ?/?`, `# ??/??`) are negative
numbers when typed, by VALUE and by `+0`. NUMBERVALUE matches except on fractions, because it drops the spaces.
`-$(12)`, `$(-12)`, `$(+12)`, `$((12))`, `$(12%)`, `$()`, `$(12`, `$($12)`, `(12$)` and `£(12)` are text or #VALUE!.
`€(12)` is text when typed but -12 by coercion, because `€` is a coercion currency, not a typed one in en-US.
Typed `$(1 1/2%)` is text with `0.00%`.
`parse_percent_or_currency` reads a currency-prefixed, parenthesised, unsigned amount as negative, and so does
`read_mixed_fraction` for a fraction. `mixed_fraction_denominator_is_wide` ignores the closing `)`.
This also fixes the script-digit `$(١٢)` (t3891 B29:C29). The rule is in docs/excel-semantics/coercion.md
"Currency sign before parentheses". Eval: probes/t4117-currency-parens passes 170/170 cells, with no
regressions. Opus resume: added rows 15-34 for the fidelity reviewer's round-2 findings (`€`, E-notation,
`%`, fractions, malformed forms) and the fraction branch.

## Handoff (resumed)
Sonnet run, PR #3200 (branch agent/engine/T-4117-currency-paren, head de90a01be). Done: probes/t4117-currency-parens (A1-A14, excel-graph) shows `$(12)`, `$ (12)`, `$( 12)`, `$(12 )`, `$( 12 )`, `$(1,234.5)`, `$(12.5)` are negative; `-$(12)`, `$(-12)`, `$((12))`, `(12$)`, `($)12`, `(12)$` are text. Code is in `parse_percent_or_currency` (currency-prefix branch), rule in coercion.md, ENTERED log in docs/research/t4117-currency-parens-2026-11-04.md. Design and security APPROVE on de90a01be. Reviewer-excel-fidelity requested changes twice (round 2, still open):
- MAJOR: the branch runs for every currency in `currencies` (`€`, locale symbol), not just `$`; only `$` is probed. Probe `€(12)`, `€ (12)`, `€(1,234.5)` (typed, VALUE, NUMBERVALUE, +0) or restrict to `$`, and make coercion.md match.
- MAJOR: the amount goes through full `parse_number`, so `$(1e3)`, `$(12%)`, `$(1 1/2)` now parse unprobed (t3334 H39 shows `$` and E-notation differ). Add rows `$(1e3)`, `$(1.5E+2)`, `$(12%)` and implement or reject per Excel.
- MINOR: probe `$(+12)`, `$()`, `$(12`; use one trim (`trim_matches(' ')`) for both the sign check and the cut.
Next: add those rows, refill the oracle (`rustsheet-oracle fill -b excel-graph --suite probes`; capture the ENTERED lines into the research note), adjust the code, `make eval-baseline`, `make check-fast`, rerun the fidelity reviewer. Set `status: done` and refresh the Outcome (it now says 14 rows) when finished.
