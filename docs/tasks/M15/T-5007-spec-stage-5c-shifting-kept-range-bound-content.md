---
id: T-5007
title: "Spec: stage 5c, shifting kept range-bound content and drawing anchors under row and column edits"
milestone: M15
lane: research
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-5006]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A spec for keeping range-bound content under row and column edits by shifting it rather than dropping it. It covers
known extension shapes (x14 data validation `xm:sqref`, sparklines, x14 conditional formats) and drawing anchors.
This is research §8 items 28 and 29, stage 5c.

## Context
Decision 0275 D4 drops stale content. Stage 5c narrows what counts as stale by teaching the shifter the shapes it
can rewrite exactly as Excel does. It reuses M4's `RefMap`, with no second reference adjuster. Each shape needs
oracle evidence of how Excel rewrites it.

## Acceptance criteria
- [ ] SPEC-0034 gains a stage 5c section: the shapes, the evidence each needs, and what still drops.
- [ ] The planner files its implementation tasks.
- [ ] Reviewers approve.

## Out of scope
- Implementation.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
