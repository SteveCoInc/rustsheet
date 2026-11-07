---
id: T-4922
title: "A missingCaption of a lone minus sign renders as 0"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
With `showMissing` and a `missingCaption` of `-`, empty intersections render as the number 0 in the data field's number format, as Excel does.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

Every gen-pivot pivot whose missingCaption is `-` shows 0 in its empty intersections (shown `0`, or `0.00` under a #,##0.00 data field), never `-`: gen-pivot/ca5c8fa99c13d747 P01!O9:AJ28, gen-pivot/a6d74990f3ae755e P07 and P09, gen-pivot/ec473260bb2065b3 P03, P09 and P12, gen-pivot/2251b0f5e5a256e1 P04. The pivots whose caption is `n/a` show the caption (gen-pivot/f9e7edbb5b74011a P08, gen-pivot/462f10551d480f9d P12, gen-pivot/01865529b5a09c0b P10), as probes/t4137-pivot-captions K2 does. The engine writes `-`. Which captions Excel reads as numbers (`0`, `-0`, ` - `, `1`) is not probed: the implementation should follow only what is evidenced and the task should add a probe for the rest.

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/layout/render.rs (missing caption). The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4918, T-4919, T-4920, T-4921, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 1175 values and texts on 8 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/2251b0f5e5a256e1 P04!O10:W66
- gen-pivot/a6d74990f3ae755e P07!O9:Z33
- gen-pivot/a6d74990f3ae755e P09!M10:N96
- gen-pivot/ca5c8fa99c13d747 P01!O9:AJ28
- gen-pivot/ca5c8fa99c13d747 P05!M8:S11
- gen-pivot/ec473260bb2065b3 P03!N8:S10
- gen-pivot/ec473260bb2065b3 P09!N10:AZ23
- gen-pivot/ec473260bb2065b3 P12!N9:P19

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
