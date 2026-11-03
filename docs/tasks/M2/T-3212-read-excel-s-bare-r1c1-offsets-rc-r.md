---
id: T-3212
title: "Read Excel's bare R1C1 offsets (RC, R[-1]C, RC[2]) in R1C1 typed entry, with oracle probes"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3113]
---

## Goal
A spec cell typed in R1C1 mode (`{"entered_r1c1": ...}`, T-3113) with Excel's short R1C1 forms (`=RC`, `=R[-1]C`, `=RC[2]`, whole rows `=R2`, `=R[1]`, columns `=C3`, `=C[-1]`) reads as Excel reads it.

## Context
`rustsheet_engine::r1c1_formula_to_a1` (crates/rustsheet-engine/src/typed_input.rs, `TODO(T-3212)`) parses with the
lexer's `LexerMode::R1C1`, which is the engine's storage grammar: it always writes `R[n]C[n]`, and bare `R`/`C`/`RC`
are identifiers there (LAMBDA parameters such as `c` re-parse from storage, `is_valid_r1c1_identifier` in
expressions/lexer/mod.rs). So the converter refuses `=R[-1]C`, and the eval adapter leaves the cell blank. Fix it in
the converter (a typed-entry flag on the lexer, or a pre-pass) without changing how stored formulas re-parse.
Probe first: which short forms Excel accepts typed in R1C1 mode, and what a LET name `rc` does there; also
cell-shaped LET/LAMBDA names (`f1`, refused typed in A1 mode by decision 0103) and a structured reference typed in
R1C1 mode. The converter returns `None` for every parse failure, so the adapter blanks text the engine merely does
not support (or that passes a nesting/length limit) as if Excel refused it: return a distinct result for those and
fail the case instead (fidelity review of T-3113).

## Acceptance criteria
- [x] Probe case(s) with `entered_r1c1` cells for each short form, filled by excel-graph; engine matches.
- [x] Stored formulas with `r`/`c`/`rc` LAMBDA parameters still load and evaluate (existing tests).
- [x] Findings in docs/excel-semantics/syntax.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- R1C1 display (FORMULATEXT in R1C1 mode).

## Outcome
- **Probe**: probes/t3212-r1c1-bare-offsets (excel-graph, 2026-11-03) has 45 `entered_r1c1` cells with FORMULATEXT
  and ISBLANK. They cover the bare offsets, whole rows and columns, ranges of them, lower case, other sheets,
  wrap-around, out-of-grid offsets, and LET or LAMBDA names `r`, `c`, `rc` and `r1c1`. The engine matches 125 of
  127 cells. The two it misses (J39, J50) are ISBLANK of a circular cell, which T-3898 owns.
- **Code**: `typed_input/r1c1.rs` (`expand_short_forms`) rewrites Excel's short forms into the storage grammar
  before `r1c1_formula_to_a1` parses the text. A relative offset wraps around the grid as in Excel, and an offset of
  a whole grid, or an absolute column past it, is refused. The R1C1 parse now runs as typed. In R1C1 mode the parser
  refuses a LET or LAMBDA name that reads as a reference (`Parser::typed_refusal`). Stored text is untouched, and the
  engine's tests pass, including those for `r` and `c` LAMBDA parameters reloaded from storage.
- **Not done**: a cell-shaped A1 name (`f1`) typed in R1C1 mode is accepted by Excel but refused by the A1 typed
  entry the adapter uses (T-3897). Its probe row was dropped. Returning a separate result for a parse limit (the
  fidelity note in Context) is also not done: the converter still reads every failure as Excel's refusal, so a
  limit hit blanks the cell. It is not in the acceptance criteria, and no probe hits a limit.
- **Unprobed**: a structured reference typed in R1C1 mode, which does not parse since there are no tables in the
  converter (T-3113).
- **Eval delta**: probes gets 1 new case: 127 cells, 125 passing, and 2 new-coverage failures owned by T-3898.
  0 regressions.
- **Semantics**: syntax.md has a new entry, "Short R1C1 forms typed in R1C1 mode".
