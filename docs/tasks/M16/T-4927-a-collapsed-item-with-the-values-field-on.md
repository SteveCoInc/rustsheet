---
id: T-4927
title: "A collapsed item with the Values field on rows keeps one line per data field"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A collapsed row item with the Values field inside it keeps one line per data field below the item, as Excel lays it out.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

gen-pivot/36996978427905b2 P05 (compact, rows Region, Rep, Product, Values; West collapsed; Sum of Sales and Count of Score): Excel shows `West` on L6 with `Sum of Sales` 3 and `Count of Score` 2 on L7:L8, then East and its children; the engine puts the first data field's value on West's own line and drops the data lines, so every later line is two rows up (location L5:M31 in Excel, L5:M29 in the engine). pivots.md "Layout lines" had the engine's one-line reading UNVERIFIED (T-4699).

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/layout/lines.rs (`collapsed`). The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4918, T-4919, T-4920, T-4921, T-4922, T-4924, T-4925, T-4926); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 48 values and texts on 1 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/36996978427905b2 P05!L6:M31

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
