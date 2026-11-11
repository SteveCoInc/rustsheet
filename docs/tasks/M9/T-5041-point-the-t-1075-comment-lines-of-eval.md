---
id: T-5041
title: "Point the T-1075 comment lines of eval/milestones exception tsvs at T-4373 (needs a decision record)"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
No comment line of eval/milestones/exit-exceptions.tsv or phase1-exceptions.tsv names the dropped T-1075 as owner of a cell decision 0324 gave to T-4373.

## Context
Left over from T-4646 (PR #4028): scripts/check_governance.py refuses any edit to the protected eval/milestones/ paths, comment lines included, without a decision record. Lines: exit-exceptions.tsv 8, 16, 21; phase1-exceptions.tsv 9-10, 20-21, 26 (repoint to T-4373, decision 0324). The record can be a short amendment of decision 0324.

## Acceptance criteria
- [ ] `rg -n T-1075 eval/milestones` finds no owner reference; no data row changed.
- [ ] A decision record covers the protected-path edit (`make new-decision`).
- [ ] `make check` green; reviewers approve.

## Out of scope

- Any change to which cells are listed.
