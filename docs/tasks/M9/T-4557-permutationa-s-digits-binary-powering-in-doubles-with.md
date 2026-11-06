---
id: T-4557
title: "PERMUTATIONA's digits: binary powering in doubles with decision 0188 R4's hold-out and fresh sweep (T-4501 split)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
PERMUTATIONA's numeric cells are Excel's digits, if right-to-left binary powering in doubles holds on fresh points.

## Context
Split from T-4501, which shipped only the number_chosen `#NUM!` cut (semantic). statistics.md "PERMUTATIONA is binary
powering in doubles, and #NUM! at a huge number_chosen (T-4106)": binary powering gives 104 of the 105 numeric cells of
accuracy/PERMUTATIONA/s1 bit for bit against 73 for the engine's `powf`. The model was chosen by scoring candidates against
those cells, so decision 0188 R4 applies: a hold-out split fixed before scoring, a fresh log-uniform sweep filled after the
model is frozen, and the Gate A envelope on it. Every cell is already inside Gate A's envelope, so this is last-digit work
(Phase 1b). `PERMUTATIONA(10,308)` = 1E+308 exactly is unexplained.
Code: `permutationa_of`, crates/rustsheet-engine/src/functions/statistical/permut.rs.

## Acceptance criteria
- [ ] A fresh sweep (excel-graph) filled after the model is frozen confirms binary powering beats `powf` on held-out
      points, or the Outcome records that it does not and nothing changes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The number_chosen cut (T-4501).
