---
id: T-4646
title: "Point the stale T-1075 references at T-4373 and decision 0324"
milestone: M9
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
No text names the dropped T-1075 as the owner of a cell that decision 0324 gave to T-4373.

## Context
Review follow-up of T-4534 (PR #3681, MINOR findings from the design and excel-fidelity reviewers). Decision 0324 moved the
owner column of the former T-4534 rows of eval/milestones/exit-exceptions.tsv and phase1-exceptions.tsv to T-4373, and
added poi EverythingTests!J1328. Some prose still names T-1075, which decision 0266 dropped:
- the comment lines of eval/milestones/exit-exceptions.tsv (IF_ARRAY, COMPLEXs, IRR/K25 blocks: "their M14 item is
  T-1075 ...") and of eval/milestones/phase1-exceptions.tsv (J136/MODE, COMPLEXs, IRR/K25 blocks). eval/milestones/ is
  protected (decision 0044): editing comment lines only needs this task to cite decision 0324 in the PR. If the hygiene
  gate refuses an edit without a new decision, record that and skip the tsv comments.
- docs/excel-semantics/coercion.md: the SUMPRODUCT J1328 entry and the MODE G936/H936/K936 entry ("until a 16.x desktop
  value decides (T-1075, M14)").
- `rg -n "T-1075" docs/excel-semantics eval/milestones` for any others.

## Acceptance criteria
- [ ] Each reference above names T-4373 (and decision 0324 where it explains the owner), or the PR says why it stays.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any change to which cells are listed, or to their decisions.
