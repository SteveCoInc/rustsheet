---
id: T-4542
title: "Review MINORs of #3538: numeral_suffix doc, French regional names claim, a tie on a plain engineering format"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The three MINOR findings of #3538's Excel-fidelity review are resolved: the `numeral_suffix` doc states the new rule, the French regional names claim is probed or narrowed, and a tie on a plain engineering format is probed.

## Context
From the review of #3538 (T-4062, T-4064, T-4065, T-4069, T-4492):
1. formatter/locale_tag.rs `numeral_suffix`'s doc still says "the locale part resolves as the whole tag always has". Since T-4492, `resolve_with` sends only the text before the comma, spaces trimmed, to `resolve_bcp47` (probes/t4492-comma-suffix-reading A14:A19). Reword it.
2. number-formats.md "More regional variants" says fr-SN, fr-CI, fr-ML, fr-CD, fr-HT, fr-RE, fr-MC, fr-LU, fr-CH, fr-CA and fr-BE "print as fr-FR". probes/t4065-french-regional-halves checks only their day halves and March. Probe their 12 months and 7 weekdays (as probes/t4065-french-regional-names does for four LCIDs), then fix any table or narrow the sentence.
3. `round_engineering` (decimal_round.rs) now rounds mantissa halves away from zero for every engineering format, evidenced only on grouped ones (probes/t4062-grouping-in-scientific B1:B11). Probe ties on plain engineering formats (`##0E+0` on 12.5 and 1250, `##0.0E+0` on 12.25 and 0.125, `#0E+0` on 2.5).

## Acceptance criteria
- [ ] The doc comment states the rule; the two probes are filled and pass (or their open cells are owned); number-formats.md matches the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
