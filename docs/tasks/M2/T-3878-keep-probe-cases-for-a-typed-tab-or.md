---
id: T-3878
title: "Keep probe cases for a typed TAB or NBSP between tokens in A1 mode and an NBSP after the bang in R1C1 mode"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle-backed probe cases pin the typed refusal of a TAB or NBSP between tokens, in A1 mode and in R1C1 mode.

## Context
- Follow-up of T-3102 (design and fidelity reviews, MINOR). Since T-3102 the engine refuses a typed formula whose
  whitespace between tokens holds a TAB or an NBSP (`Lexer::skipped_refused_whitespace`, `Parser::typed_refusal`).
  The A1-mode evidence is T-2949's probe run, whose cases were not kept (docs/excel-semantics/syntax.md "Remaining
  whitespace shapes"). The R1C1-mode NBSP refusal is inferred; only the TAB was probed
  (probes/t3102-r1c1-ws-after-bang A3).
- Use the pattern of probes/t3102-r1c1-ws-after-bang: one `entered` (or `entered_r1c1`) cell per shape, plus
  `=ISBLANK(...)` and `=IFERROR(FORMULATEXT(...),"none")` beside it. Shapes: `=1+` TAB `2`, `=` TAB `1+2`, `=1` TAB `+2`,
  `=1+` NBSP `2`, `=S!` NBSP `D1` (A1 mode), `=S!` NBSP `R1C4` (R1C1 mode), and `=LEN("a` TAB `b")` as the control.

## Acceptance criteria
- [ ] Probe case filled on excel-graph; the engine matches every cell (or a task owns each visibly-off one).
- [ ] Reflow the `Parser::typed_refusal` doc comment in crates/rustsheet-engine/src/expressions/parser/operands.rs
      (T-3102 design review, MINOR: the list reads "..., or `ANCHORARRAY(` ..., or a TAB").
- [ ] syntax.md cites the case and drops "cases not kept" / "inferred" for the shapes it covers.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
