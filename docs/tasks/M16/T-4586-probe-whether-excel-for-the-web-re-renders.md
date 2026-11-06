---
id: T-4586
title: "Probe whether Excel for the web re-renders a pivot without refreshOnLoad on open, and after a source edit"
milestone: M16
lane: research
status: todo
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
docs/excel-semantics/pivots.md states, with filled probes, two things about a pivot without `refreshOnLoad`: whether Excel for the web re-renders it on open, and whether it re-renders a pivot after a source edit with no refresh.

## Context
SPEC-0032 Goal 3 assumes that a pivot without `refreshOnLoad` keeps the cells the file saved, and that a source edit does not re-render. Every rendered probe so far (t4123, t4128, t4134, t4137, t4138) is `refreshOnLoad` (pivot_definition.py), so neither is evidenced (Excel-fidelity review of T-4121). Cases:
- (a) a pivot with `refreshOnLoad=0` whose saved cells and `rowItems` disagree with its cache;
- (b) the same pivot with no `rowItems`;
- (c) a `refreshOnLoad` pivot whose source cell a `check` formula reads after the oracle edits the source without a refresh (Graph range write, then a read of the pivot area).

The builder needs a `refreshOnLoad` switch. T-4182 (Auto Refresh) builds on (c).

## Acceptance criteria
- [ ] Filled probes for (a), (b) and (c); pivots.md entries for each; SPEC-0032 Goal 3 updated if Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine behaviour (T-4151).
