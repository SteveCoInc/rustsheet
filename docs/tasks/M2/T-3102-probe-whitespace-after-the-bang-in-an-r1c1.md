---
id: T-3102
title: "Probe whitespace after the bang in an R1C1-mode formula (formulasR1C1) and what we do with S! R1C2 typed in A1 mode"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-3113]
---

## Goal
Oracle evidence for whitespace after `!` in a formula entered in R1C1 mode (`S! R1C2`, via `formulasR1C1`), and what RustSheet does with `=S! R1C2` typed in A1 mode (Excel refuses it, 400 InvalidArgument); adjust the lexer if it differs.

## Context
T-3113 added the R1C1 entry mode (`{"entered_r1c1": "=S! R1C2"}`, decision 0179) and records a typed entry Excel refuses as a blank cell (probe it with `=ISBLANK(A1)`); the engine's R1C1 reader does not take bare `R`/`C` offsets yet (T-3212), so use `R1C2`-style references. T-2869 probed the A1-mode forms only (docs/excel-semantics/syntax.md, "drop after `!`"). Fidelity-review follow-up.

## Acceptance criteria
- [x] Probe case(s) with oracle values; engine matches; syntax.md entry updated.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Handoff (resumed)
Tried: scoped the probe. Blocker found: no spec cell can enter R1C1-mode text today. `{"entered": ...}` is
written by the excel-graph backend as a `values` PATCH (tools/src/rustsheet_tools/spec2xlsx.py `entered_cells`
via docProps `rustsheet.entered.<n>`, tools/src/rustsheet_tools/oracle/backends.py line ~84, docs/architecture/oracle.md
"Typed input"), and `rg formulasR1C1` finds nothing in tools/ or crates/. The probe needs a new entry mode
(e.g. `{"entered_r1c1": "=S! R1C2"}` written as a `formulasR1C1` PATCH), plumbed through spec2xlsx.py, the
backend, `CellInput` in crates/rustsheet-eval/src/case.rs and the engine adapter (engine.rs ~491), plus a
decision on whether the engine adapter should enter it in R1C1 mode. That is a schema/design change across
Python and Rust, beyond a sonnet S task.
Next: add the entry mode (own spec/decision if it changes the case schema), then write the probes
(`=S! R1C2`, `=S!`+LF+`R1C2`, tab variants in R1C1 mode; `=S! R1C2` in A1 mode expecting 400), fill with
excel-graph (status OK today), update syntax.md "drop after `!`" and adjust the lexer if it differs.

## Outcome
- Probe `probes/t3102-r1c1-ws-after-bang` (excel-graph 2026-11-02, the `entered_r1c1` mode of T-3113, so the handoff's
  blocker is gone): typed in R1C1 mode, `=S! R1C4`, `=S!` LF `R1C4` and `=S!  R1C4+1` read S!D1 and print `=S!$D$1`
  (whitespace dropped, as in A1 mode); `=S!` TAB `R1C4` is refused (PATCH 400); `=S! R1C4` typed in A1 mode is refused.
- Ours differed only on the TAB: it read it. Fix: the lexer records a TAB or NBSP in the whitespace it skips between
  tokens (`Lexer::skipped_refused_whitespace`), and a typed formula holding one is refused as one that does not parse
  (`Parser::typed_refusal`, `TYPED_FORMULA_DOES_NOT_PARSE`), in A1 mode too, which is T-2949's unkept evidence
  (`=1+` TAB `2`, `=` TAB `1+2`, NBSP forms; syntax.md "Remaining whitespace shapes"). `r1c1_formula_to_a1` refuses it
  as well, because the A1 text it prints has no whitespace left. Stored text still loads (a TAB is dropped).
- Tests: `typed_tab_or_nbsp_between_tokens_is_refused` (engine). syntax.md "drop after `!`" and "Remaining whitespace
  shapes" record the findings.
- Eval delta: +16 cells new coverage (probes/t3102-r1c1-ws-after-bang, all pass); regressions 0.
