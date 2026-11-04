---
id: T-4117
title: "Typed $(12): a currency sign before parentheses is a negative number in Excel ($(١٢) is -12), text in ours"
milestone: M5
lane: engine
status: in-progress
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
- [ ] probes/t3891-typed-script-digit-shapes B29:C29 and the new probe's cells pass; the rule is in
      docs/excel-semantics/coercion.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Script-digit tags (T-3891).

## Outcome
Probed ASCII forms (probes/t4117-currency-parens, excel-graph): `$(12)`, `$ (12)`, `$(1,234.5)`, `$(12.5)` are negative numbers by typing, VALUE, NUMBERVALUE and `+0`; `-$(12)`, `$(-12)`, `(12$)`, `($)12`, `(12)$` are text/#VALUE!. `parse_percent_or_currency` now reads a currency-prefixed parenthesised unsigned amount as negative, which also fixes the script-digit `$(١٢)` (t3891 B29:C29). Rule in docs/excel-semantics/coercion.md. New probe: the new probe cells pass.

## Handoff (resumed)
Sonnet run, PR #3200 (branch agent/engine/T-4117-currency-paren, head de90a01be). Done: probes/t4117-currency-parens (A1-A14, excel-graph) shows `$(12)`, `$ (12)`, `$( 12)`, `$(12 )`, `$( 12 )`, `$(1,234.5)`, `$(12.5)` are negative; `-$(12)`, `$(-12)`, `$((12))`, `(12$)`, `($)12`, `(12)$` are text. Code is in `parse_percent_or_currency` (currency-prefix branch), rule in coercion.md, ENTERED log in docs/research/t4117-currency-parens-2026-11-04.md. Design and security APPROVE on de90a01be. Reviewer-excel-fidelity requested changes twice (round 2, still open):
- MAJOR: the branch runs for every currency in `currencies` (`€`, locale symbol), not just `$`; only `$` is probed. Probe `€(12)`, `€ (12)`, `€(1,234.5)` (typed, VALUE, NUMBERVALUE, +0) or restrict to `$`, and make coercion.md match.
- MAJOR: the amount goes through full `parse_number`, so `$(1e3)`, `$(12%)`, `$(1 1/2)` now parse unprobed (t3334 H39 shows `$` and E-notation differ). Add rows `$(1e3)`, `$(1.5E+2)`, `$(12%)` and implement or reject per Excel.
- MINOR: probe `$(+12)`, `$()`, `$(12`; use one trim (`trim_matches(' ')`) for both the sign check and the cut.
Next: add those rows, refill the oracle (`rustsheet-oracle fill -b excel-graph --suite probes`; capture the ENTERED lines into the research note), adjust the code, `make eval-baseline`, `make check-fast`, rerun the fidelity reviewer. Set `status: done` and refresh the Outcome (it now says 14 rows) when finished.
