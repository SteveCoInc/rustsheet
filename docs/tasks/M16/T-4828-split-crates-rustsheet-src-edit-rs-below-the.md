---
id: T-4828
title: "Split crates/rustsheet/src/edit.rs below the 1000-line limit (pivot and table-edit plumbing into edit/)"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
crates/rustsheet/src/edit.rs has room to grow: its pivot and table-edit plumbing moves into modules under crates/rustsheet/src/edit/, and edit.rs stays well below the 1000-line file-size limit, with no behaviour change.

## Context
edit.rs is 994 lines after T-4152, so the next API change (T-4178, T-4184) would fail scripts/check_file_size.py. T-4669 split workbook.rs and filed this as its sibling, as T-4669's context allowed. edit/ already holds pivot.rs; follow the workbook/ pattern of one `impl` block per module, moved verbatim.

## Acceptance criteria
- [ ] edit.rs is under 700 lines; no public item changes path or signature; test bodies unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Behaviour changes.
