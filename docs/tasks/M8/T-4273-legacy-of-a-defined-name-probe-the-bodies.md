---
id: T-4273
title: "Legacy @ of a defined name: probe the bodies T-3856 left to engine choice (relative cell, negative number, error literal, parenthesized, name of a name, INDEX over an array-bodied name) and match"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A legacy (no `cm`) formula that uses a defined name shows the `@` (in FORMULATEXT and on export) and evaluates as Excel
does for the name bodies docs/excel-semantics/references.md still marks as engine choices, with excel-graph evidence for
each.

## Context
- references.md "A defined name takes the `@` by its body" (T-3856, probes/t3856-legacy-at-name-body, 261 cells) ends:
  "over an array-bodied name, where `INDEX(n2d,1)` with n2d = `{1,2;3,4}` is a row: ours keeps `=@INDEX(n2d,1)`,
  unprobed" and "Unprobed: a relative one-cell body (ours: a cell, no `@`), and a negative number (`=-3`), an error
  literal, a parenthesized body or a body that is another name (ours: the `@`; spaces around one token are trimmed)".
  It also says "whether Excel redraws a legacy formula's `@` when a name's body changes later is unprobed".
- Bodies to probe, each at the root, under ABS, under SUM, as an IF arm and as an INDEX index (t3856's columns), with
  FORMULATEXT beside each: `nRel` = Sheet1!Z3 (relative, defined from a cell on row 30), `nNeg` = -3, `nErr` = #N/A,
  `nPar` = (3), `nPad` = ` 3 ` (spaces around the token), `nName` = nNum (a name whose body is another name, both one
  literal), `nNameRng` = nRng (a name of a range-bodied name), and `=INDEX(n2d,1)`, `=INDEX(n2d,1,1)` with
  n2d = {1,2;3,4}.
- The redraw question needs a body changed after the legacy formula is read; the case format sets names before load,
  so it is out of reach here unless the harness gains a name edit op (do not add one in this task; note it in the
  Outcome).
- Code: `scalar_body` (crates/rustsheet-engine/src/expressions/parser/static_analysis/name_body.rs) decides which bodies
  read as their literal or cell; `indexes_one_reference_cell` (static_analysis/if_arm.rs) and `fallback_shape`
  (expressions/parser/static_analysis.rs) decide the single-cell INDEX shape; `add_legacy_intersection` (same file)
  places the `@`.
- Not the same as T-4212 (todo, sonnet), which probes a single-cell INDEX over a union, a sheet-qualified or 2-D range and
  a reference-bodied name; this task's INDEX cells are over an array-bodied name only. Keep the probe cases separate.
- New probe case: `t4273-legacy-at-name-body-more`.

## Acceptance criteria
- [ ] probes/t4273-legacy-at-name-body-more is filled on excel-graph and every cell passes in `make eval`, with no new
      F line in eval/baseline/probes.txt and probes/t3856-legacy-at-name-body still passing.
- [ ] references.md replaces the two "unprobed" clauses above with the probed rule (VERIFIED, cells cited); the redraw
      question stays listed as unprobed with its reason.
- [ ] Unit tests in the engine's legacy-@ tests cover each body kind's FORMULATEXT.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The single-cell INDEX area kinds of T-4212.
- A name-edit sheet op for the harness (file a harness task if the redraw question is wanted).
