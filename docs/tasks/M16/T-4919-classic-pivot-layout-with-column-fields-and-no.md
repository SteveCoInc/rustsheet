---
id: T-4919
title: "Classic pivot layout with column fields and no row field"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Classic layout (`gridDropZones`) with column fields and no row field renders as Excel does: the data field's caption at the location's corner and a `Total` label beside the value row.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

With gridDropZones, no row field and one data field (gen-pivot/01865529b5a09c0b P08, outline: Excel L5 `Average of Score`, L8 `Total`; the engine leaves L5 blank and puts `Average of Score` at L8), and with two data fields (gen-pivot/2251b0f5e5a256e1 P07!L9, gen-pivot/640780d7b6f4ad53 P11!L9, gen-pivot/ec473260bb2065b3 P02!L9: Excel `Total`, engine blank). The engine's own readings for these shapes were UNVERIFIED in pivots.md "Classic layout (gridDropZones)" (probe task T-4785). Related: T-4785 probes classic layout more widely; coordinate so the two are not worked at cross purposes.

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/layout/render.rs (`render_within`, `CLASSIC_TOTAL`), crates/rustsheet-engine/src/pivot/layout/lines.rs (`geometry`). The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4918, T-4920, T-4921, T-4922, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 5 values and texts on 4 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/01865529b5a09c0b P08!L5:L8
- gen-pivot/2251b0f5e5a256e1 P07!L9
- gen-pivot/640780d7b6f4ad53 P11!L9
- gen-pivot/ec473260bb2065b3 P02!L9

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
