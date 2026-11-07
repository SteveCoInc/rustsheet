---
id: T-5045
title: "Paste keeps pasted content in covered cells of a recreated merge (SPEC-0036 S2)"
milestone: M11
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-5044]
---

## Goal
A paste that recreates a merged range keeps the content it pasted into the range's covered cells.

## Context
Spec: docs/specs/SPEC-0036-merged-covered-content.md, row U5 (rollout S2). Today cut-paste (crates/rustsheet-engine/src/user_model/clipboard.rs,
~390-460) and copy-paste (~630-670) write the pasted cells, then recreate each source merge with `merge_cells_keep_styles` and
`covered_cells_clear_diffs`, which clears every covered position, wiping covered content the paste just wrote. Change: give both a
`kept` area (the pasted block) so covered cells inside it keep their content and get no clear diff; only cells a recreated merge
swallows outside the pasted block are cleared. Factor the two copies of the merge-recreation code into one helper. Excel's paste over
covered content is UNVERIFIED and not probeable on excel-graph (SPEC-0036 X9).

## Acceptance criteria
- [ ] Copy-paste and cut-paste of a merged range with covered content reproduce the content at the target; undo and redo restore
      both ends exactly (tests in crates/rustsheet-engine/src/test/user_model/test_paste_merged_cells.rs).
- [ ] One merge-recreation helper serves both paste paths.
- [ ] docs/excel-semantics/xlsx-storage.md's covered-content entry records the paste rule as UNVERIFIED (design choice, SPEC-0036).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Merge (U2) and autofill (T-5047).
