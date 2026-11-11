---
id: T-90006
title: "AGGREGATE array form over a 3-D span gives the #VALUE! Excel gives"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
usage: foundational
depends_on: []
---

## Goal
AGGREGATE's array form (14-19) over a 3-D span gives Excel's #VALUE!.

## Context
Owns: probes/aggregate-array-form-3d-span-hidden (every visibly-off cell).
- Excel gives #VALUE! for `AGGREGATE(14,5,First:Last!A1:A9,1)`, (15,5,Mid:Last..,10), (16,..,0.5), (14,4,..), the LET form and (19,5,..); ours computes across the sheets (29, 22, 19, 6.5). Excel takes no 3-D span in the array form. Follow-up of T-5292's review.
- Source: (excel-graph fill, 2026-11-11, #4125 wave 2; Excel's value first, ours second).

## Acceptance criteria
- [ ] Every visibly-off cell of probes/aggregate-array-form-3d-span-hidden matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
