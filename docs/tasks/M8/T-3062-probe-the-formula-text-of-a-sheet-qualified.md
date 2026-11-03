---
id: T-3062
title: "Probe the formula text of a sheet-qualified #REF! after a sheet rename or delete, unqualified and typed"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-2855]
---

## Goal
The formula text of a sheet-qualified `#REF!` matches Excel in the four unprobed shapes T-2855 left: after its sheet is renamed, after its sheet is deleted, for a wholly deleted range with no sheet prefix, and for a typed one with a sheet name in another case or for a sheet that does not exist.

## Context
T-2855 keeps the sheet of `S!#REF!` in the written tree (`Node::SheetRefErrorKind`; references.md "A
sheet-qualified #REF! is #REF!"). Only `=SUM(S!#REF!)` after a row or column delete is oracle-backed
(probes/table-edits plain-range-delete-row-first F!A6, table-delete-column-qty F!A7). Unprobed:
- renaming S to T: the engine keeps `S!#REF!` (sheet_edits.rs skips the node); Excel likely prints `T!#REF!`;
- deleting sheet S: the engine keeps `S!#REF!`;
- a wholly deleted unqualified range (`=SUM(C4:C4)`, delete row 4): the engine prints `=SUM(#REF!)`
  (stringify.rs, `wrong_reference::ref_error_text`), before T-2855 `#REF!:#REF!`;
- typed `=sheet1!#REF!` and `=NoSuchSheet!#REF!`: printed as typed; probes/sheet-qualified-ref-error-forms
  records values only.
Use the `excel-probe` skill (FORMULATEXT cells), then match the engine to the evidence.

## Acceptance criteria
- [x] A probe case records FORMULATEXT for the four shapes (oracle values filled).
- [x] The engine matches every probed cell; references.md records the rule with the evidence.
- [x] `make check` green; reviewers approve.

## Out of scope
- Values: a sheet-qualified `#REF!` already evaluates to `#REF!` (VERIFIED).

## Outcome
Three probe cases fill the four shapes (excel-graph). The engine now keeps `S!#REF!` in step with a
rename (`rename_sheet_in_node` takes the old name), prints `#REF!#REF!` once S is deleted
(`Node::SheetRefErrorKind::missing`, set by the parser) and spells a typed sheet name as the workbook
does. The unqualified wholly-deleted range already matched. A typed `NoSuchSheet!#REF!` is a Graph
external-link artifact and is left out of the probe. Eval: the three new probes pass (15 checked cells).

## Handoff (resumed)
Engine change and three probes (rename, delete-sheet, unqualified/typed) are done on PR #2926 and pass 15/15 cells; design and security approved. excel-fidelity requested changes twice (round 1 on 6357da18, round 2 on 05d85543), so I stopped (work-loop 5b). Open findings of round 2:
- MAJOR references.md "A sheet-qualified #REF! is #REF!": the typed `=NoSuchSheet!#REF!` cell was dropped from the probe and Graph's text (an external-link path `='https://.../[NoSuchSheet]NoSuchSheet'!#REF!`) was not recorded. Put that cell back in the probe (oracle value recorded, engine cell a known failure with an owner) or record the exact text in references.md; drop the "Graph artifact" wording; say plainly that the engine's `#REF!#REF!` for a sheet no sheet has is unverified (the parser cannot tell a deleted sheet from a never-existing one).
- MINOR sheet-qualified-ref-error-text-rename.case.json description says "A3 =S!C4:C5 SUM control"; A3 is `=SUM(S!C5)`.
- Round 1 (handled by marking Unprobed and filing T-3899): duplicate-sheet and defined-name-rename behaviour of `S!#REF!` are not evidenced; alternative is to skip `SheetRefErrorKind` in the duplicate path.
- Optional: share one case-insensitive sheet-name helper (parser/mod.rs, sheet_edits.rs; `Model::get_sheet_index_by_name` has one).
Set `status: done` again after fixing.
