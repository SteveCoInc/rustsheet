---
id: T-90002
title: "Typed reference to a missing sheet is #BLOCKED! on Excel for the web (t5313 stress inputs)"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel for the web's `#BLOCKED!` for a formula typed with a reference to a sheet that does not exist is explained
(what it is, and what desktop gives), recorded in docs/excel-semantics/references.md, and an implementation task is
filed if the engine should follow it.

## Context
Owns: probes/t5313-stress-session-inputs (every visibly-off cell).
- probes/t5313-stress-session-inputs S!A37: `=Nope!A1`, typed as one entry after load into a workbook with no sheet
  named Nope (excel-graph fill, 2026-11-10). Excel for the web gives `#BLOCKED!`; ours `#REF!`.
- `#BLOCKED!` is the error Excel shows for a blocked external link. Typed `Nope!A1` names no sheet of the workbook,
  so the web may have read it as an external reference ([Nope]...). Probe that before reading anything into it: the
  same typed in a workbook that has a sheet Nope, `='Nope'!A1`, `=[Book2]Nope!A1`, and `=ISREF(Nope!A1)` and
  `=ERROR.TYPE(Nope!A1)` beside it.
- The case's request (probe ids T-5313) comes from another repository's task list, not this one (decision 0360).

## Acceptance criteria
- [ ] A probe case answers the questions above on excel-graph, and the finding is in docs/excel-semantics/references.md.
- [ ] If the engine should change, an implementation task is filed; otherwise the cell's verdict is recorded.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
