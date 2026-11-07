---
id: T-4918
title: "Leave a pivot value cell blank when its records hold no value the aggregate reads"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A pivot value cell whose records hold no value its aggregate reads (no number for Sum, Max, Var...; no non-blank value for Count) is blank, as Excel shows it.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

Excel leaves such cells blank where the engine gives the worksheet functions' answers: Count of Score over records whose Score is blank (gen-pivot/01865529b5a09c0b P01!O20, O22: engine 0), CountNums (gen-pivot/2251b0f5e5a256e1 P01!N6: engine 0), Var over records with no number (gen-pivot/f9e7edbb5b74011a P01!O9: engine #DIV/0!), Max (gen-pivot/a6d74990f3ae755e P01!R13:T16: engine 0). This answers the question docs/excel-semantics/pivots.md "Aggregates over text, blanks, booleans and errors" leaves to probe task T-4633 (a cell whose records hold no number).

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/accumulate.rs (`Accumulator::finish`), crates/rustsheet-engine/src/pivot/result.rs. The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4919, T-4920, T-4921, T-4922, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 96 values and texts on 12 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/01865529b5a09c0b P01!O20:O22
- gen-pivot/2251b0f5e5a256e1 P01!N6
- gen-pivot/2251b0f5e5a256e1 P04!T24:W50
- gen-pivot/2251b0f5e5a256e1 P10!P15:Q15
- gen-pivot/640780d7b6f4ad53 P05!T13
- gen-pivot/a6d74990f3ae755e P01!R13:T16
- gen-pivot/ec473260bb2065b3 P05!M28:P178
- gen-pivot/ec473260bb2065b3 P06!Q10:V35
- gen-pivot/ec473260bb2065b3 P07!N11:V14
- gen-pivot/ec473260bb2065b3 P09!Q12:AZ23
- gen-pivot/f9e7edbb5b74011a P01!O9:W9
- gen-pivot/f9e7edbb5b74011a P09!M8

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
