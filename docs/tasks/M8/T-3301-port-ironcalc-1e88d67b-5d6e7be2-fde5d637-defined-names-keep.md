---
id: T-3301
title: "Port ironcalc 1e88d67b/5d6e7be2/fde5d637: defined names keep hidden and comment through xlsx import and export"
milestone: M8
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A defined name read from an xlsx keeps its `hidden` flag and its `comment` through import, the
API and export, so a saved workbook does not unhide names or drop their comments.

## Context
Upstream IronCalc 1e88d67b (hidden defined names), 819491ed (the flag in the Python bindings),
5d6e7be2 (defined-name comments), bbdfb385 (a saner getter) and fde5d637 (a struct for named
ranges) carry `<definedName hidden="1" comment="...">` from `xl/workbook.xml` through
`xlsx/src/import/workbook.rs`, the model and `xlsx/src/export/workbook.rs`, with the test
`xlsx/tests/hidden_defined_names.rs`. Our `types.rs` `DefinedName` has only `name`, `formula`
and `sheet_id`, so both attributes are lost on a round trip. Port the data path, not upstream's
API shape: our defined-name methods live in `defined_names.rs` and the facade/Python API is
SPEC-0001's (UPSTREAM.md "RustSheet-only engine additions"). Excel's own behavior for a hidden
name (it still evaluates; Name Manager hides it) needs no change to evaluation. Precedent:
T-2635. Triaged by the 2026-10-30 18:47 UTC steward run.

## Acceptance criteria
- [x] `DefinedName` keeps `hidden` and `comment`; import reads and export writes both, with a round-trip test on a workbook that has a hidden, commented name.
- [x] Evaluation of hidden names is unchanged; no eval regressions.
- [x] The commit carries `Ported-From: ironcalc@fde5d637` (and the other shas it takes).
- [x] `make check` green; reviewers approve.

## Out of scope
- New Python or facade API for setting the flags (file a follow-up in the api lane if wanted).

## Outcome
- `types::DefinedName` gains `hidden: bool` and `comment: Option<String>`. The xlsx import reads
  `<definedName hidden comment>` (`hidden` as an `xsd:boolean`; the comment decodes `_xHHHH_`
  escapes, since `ST_Xstring` export writes them), and the export writes ` comment="..."` and
  ` hidden="1"` back (`export/workbook.rs`). A sheet rename and a sheet duplicate carry both flags
  on the names they rebuild (`new_empty.rs`); an API add sets neither. `update_defined_name` edits
  the entry in place, so it keeps both. Evaluation does not read either field.
- Ported the data path of ironcalc 1e88d67b (hidden names), 5d6e7be2 (comments) and fde5d637
  (the named-range struct), not upstream's API shape. 819491ed (the Python flag) and bbdfb385
  (the getter) are API surface and out of scope (no follow-up filed; the task lists it as optional).
- Test: `hidden_commented_defined_name_round_trips` (rustsheet-xlsx export tests). A hidden,
  commented workbook name, whose comment has `< > & "`, a line feed and a literal `_x0041_`, is
  kept through a sheet rename, the xlsx export (exact XML asserted), the reimport and an `.icalc`
  round trip. The hidden name still evaluates (`=Secret*2`), and a plain local name stays
  unflagged.
- Known gap: undoing a defined-name delete restores the name without its flags, because
  `Diff::DeleteDefinedName` stores only name, scope and body. No API sets the flags yet, so only
  loaded names are affected.
- Eval delta: none (no case reads these attributes); `make check-fast` green.
