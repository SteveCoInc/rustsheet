---
id: T-3865
title: "PERCENTOF: desktop probe of a defined name whose body is a 3-D span"
milestone: M14
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
PERCENTOF's result is known for `PERCENTOF(<name>,1)` when the defined name's body is a 3-D span, from desktop Excel.

## Context
T-3752 probed a 3-D span through IF, CHOOSE and a union (probes/t3752-percentof-3d-indirect; numbers.md "PERCENTOF: an error inside a reference or array is #NUM!"). The defined-name form could not be probed: excel-graph answers `FileCorruptTryRepair` (unsupportedWorkbook) for any workbook holding a name whose body is a 3-D span (`=S1:S2!$A$1`). Decision 0039 puts desktop-only questions in M14. `defined_names.rs` already treats such a body via `is_multi_sheet_span`.

## Acceptance criteria
- [ ] A probe case (desktop oracle) covers `PERCENTOF(NAME,1)`, `PERCENTOF(NA(),NAME)` and `PERCENTOF(NAME,NA())`; the engine matches each cell; numbers.md's PERCENTOF entry states the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
