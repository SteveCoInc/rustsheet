---
id: T-4637
title: "Undo of a defined-name delete restores its hidden flag and comment"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3301]
---

## Goal
Undoing the delete of a defined name that carries `hidden` or a `comment` (read from an xlsx)
puts the name back with both, so undo is a true inverse of the delete.

## Context
T-3301 added `DefinedName::hidden` and `DefinedName::comment`, kept through xlsx import, sheet
rename/duplicate and export. `Diff::DeleteDefinedName` (user_model/history.rs) stores only
name, scope and body, and its undo goes through `Model::restore_defined_name`, which builds a new
`DefinedName` with `hidden: false, comment: None` (defined_names.rs `insert_defined_name`). Found by
the design reviewer of PR #3669. Only names loaded from a file are affected, because no API sets
the flags yet. `Diff` is bitcode-encoded and can come from a peer, so the new fields need the
same untrusted-input care as the other diffs.

## Acceptance criteria
- [ ] Deleting a hidden, commented loaded name and undoing the delete restores both fields. Redo deletes the name again. A unit test covers this.
- [ ] Names added through the API are unchanged: neither flag set.
- [ ] `make check` green; reviewers approve.

## Out of scope
- An API that sets or reads the flags (api lane).
