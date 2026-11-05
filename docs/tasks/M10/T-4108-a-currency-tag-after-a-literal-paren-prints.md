---
id: T-4108
title: "A currency tag after a literal paren prints before it: ([$$-409]#,##0.00) on -12 is ($12.00) in Excel, $(12.00) in ours"
milestone: M10
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A `[$<symbol>-<tag>]` currency tag prints where it stands in the section, after any literal before it, as
Excel does.

## Context
probes/t3891-numeral-tag-render A6 (excel-graph 2026-11-04): `TEXT(-12,"[$$-ar-SA,200] #,##0.00_);[Red]([$$-ar-SA,200]#,##0.00)")`
is `($١٢.٠٠)`; ours prints `$(١٢.٠٠)`. The digits are T-3891's; the order is not: ours also prints
`$(12.00)` for `0;([$$-409]#,##0.00)` on -12, so the formatter moves the currency token ahead of the
literal `(` (lexer.rs `Token::Currency`, parser.rs, format.rs). This is the format Excel gives typed
`-$12` in a script (T-3891), so every such cell displays wrongly. Probe the LCID and bare forms first
(excel-probe): `([$$-409]#,##0.00)`, `("x"[$€-40C]0)`, `([$$]0)`, `(\$0)` on -12 and 12.

## Acceptance criteria
- [x] probes/t3891-numeral-tag-render A6 and the new probe's cells pass; the rule is in
      docs/excel-semantics/number-formats.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Numeral suffixes (T-3891, T-4107).

## Outcome
A number section's currency symbol is now inserted into the token stream where it is written (parser.rs, as date sections already did) instead of printed first. Fractions keep the old front placement. Probe probes/t4108-currency-tag-position filled from excel-graph; rule in number-formats.md.
