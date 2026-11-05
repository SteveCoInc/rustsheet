---
id: T-4271
title: "Lotus dot range with whitespace on one side only (A1 .B2, A1. B2, A1 .5): probe Excel's reading and match the lexer"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A1 formulas with a Lotus-style dot range that has whitespace on one side of the dots only (`A1 .B2`, `A1. B2`,
`A1 ..B2`) or a dot followed by a number (`A1 .5`) evaluate, print (FORMULATEXT) and load as Excel does, backed by
excel-graph evidence, where the engine today guesses.

## Context
- docs/excel-semantics/references.md "R1C1-shaped sheet and defined names" (the Lotus-range bullet): whitespace around
  the dots on both sides (`A1 . B2`) is the range (probes/lotus-dot-range-space, T-1659), but "whitespace on one side
  only, `A1 .B2`, `A1. B2`, and `A1 .5` are unprobed, and ours reads the first two as the range too". The engine's
  choice: `consume_dotted_corner` (crates/rustsheet-engine/src/expressions/lexer/ranges.rs) skips optional whitespace
  before and after the dots. In Excel a space between two references is the intersection operator, so `A1 .B2` may
  instead be refused, read as an intersection with a name `.B2`, or be the range; `A1 .5` may be an intersection with a
  number (a parse error) or a name.
- Excel for the web refuses some dotted forms at load (`FileCorruptTryRepair` for `ROWS(1..2)`, `A1.B2.C3`, T-1659's
  Outcome), so put each form in a case of its own (the lotus-dot-range-* split) so one refusal does not hide the
  others, and record a refused form in references.md as T-1659 did.
- Forms to probe, each as the value (`SUM(..)`, with the data cells of probes/lotus-dot-range), `ROWS(..)`,
  `ISREF(..)` and FORMULATEXT: `A1 .B2`, `A1. B2`, `A1 ..B2`, `A1.. B2`, `A1 .5`, `A1. 5`, and a line-feed instead of
  the space (`A1`&LF&`.B2`; LF separates intersection operands, T-0148).
- No open task covers these forms (`rg 'A1 \.B2|A1\. B2|A1 \.5' docs/tasks` finds none).
- New probe cases: `t4271-lotus-dot-one-side-*`.

## Acceptance criteria
- [ ] The probe cases above are filled on excel-graph (or each refused form is named as refused, with the Graph error),
      and every filled cell passes in `make eval` with no new F line in eval/baseline/probes.txt and no other cell
      regressing (the lotus-dot-range-* and lotus-dot-name-* cases still pass).
- [ ] references.md replaces "unprobed" for these forms with the probed reading (VERIFIED, cases and cells cited).
- [ ] Lexer tests in crates/rustsheet-engine/src/test/test_lotus_dot_ranges.rs cover each probed form.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Call-name spelling and LET-name spelling after a dotted form (settled by T-1719 and T-1720).
- R1C1-mode dotted forms.
