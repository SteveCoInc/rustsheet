---
id: T-5111
title: "FORMULATEXT shows an exponent number literal as Excel writes it (1E+300, not 1e300)"
milestone: M9
lane: engine
status: todo
priority: P3
size: S
tier: opus
usage: expert
depends_on: []
---

## Goal
FORMULATEXT (and every other place that prints a stored formula) writes a number literal with an exponent the way Excel
does: `=FVSCHEDULE(5916,{-1E+300,1E+300})` prints as written, where ours prints `{-1e300,1e300}`.

## Context
- Evidence: probes/val2-fvs-01 Probe!B3 (excel-graph, 2026-11-09): Excel's FORMULATEXT is
  `=FVSCHEDULE(5916,{-1E+300,1E+300})`; ours `=FVSCHEDULE(5916,{-1e300,1e300})` (syntax.md, "FORMULATEXT prints an
  exponent literal written `1E+300` as written").
- Unprobed, decide with a small probe before implementing (excel-probe skill): a literal typed as `1e300` or `1E300`,
  a literal outside an array constant, small exponents (`1E-5`), and whether Excel re-spells such a literal at all.
- Code: the formula stringifier behind FORMULATEXT (`rg -n 'fn_formulatext' crates/rustsheet-engine/src`); search for
  an existing number-to-formula-text helper before writing one (directive 3).

## Acceptance criteria
- [ ] probes/val2-fvs-01 Probe!B3 passes; the probe of the unprobed forms is filled and its cells pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FVSCHEDULE's value (T-5108).
