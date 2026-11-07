---
id: T-4921
title: "A multi-select page field with one visible item shows that item"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A page field whose hidden items leave one item visible shows that item, not `(Multiple Items)`, as Excel does.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

Every gen-pivot page field with several items hidden and exactly one left visible shows that item: gen-pivot/01865529b5a09c0b P05!M3 `Cy`, P11!M3 `B`, gen-pivot/2251b0f5e5a256e1 P08!M2 `Gold`; the engine shows `(Multiple Items)`. With two or more visible Excel shows `(Multiple Items)` (the page-field sheet of probes/t4137-pivot-captions). pivots.md left this UNVERIFIED (T-4657).

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/layout/render.rs (`page_area`). The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4918, T-4919, T-4920, T-4922, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 12 values and texts on 12 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/01865529b5a09c0b P05!M3
- gen-pivot/01865529b5a09c0b P11!M3
- gen-pivot/2251b0f5e5a256e1 P03!M3
- gen-pivot/2251b0f5e5a256e1 P08!M2
- gen-pivot/462f10551d480f9d P03!M3
- gen-pivot/640780d7b6f4ad53 P05!M2
- gen-pivot/a6d74990f3ae755e P01!M3
- gen-pivot/ca5c8fa99c13d747 P02!M3
- gen-pivot/ec473260bb2065b3 P05!M3
- gen-pivot/ec473260bb2065b3 P11!M3
- gen-pivot/ec473260bb2065b3 P12!M3
- gen-pivot/f9e7edbb5b74011a P12!M2

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
