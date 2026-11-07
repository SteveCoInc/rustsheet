---
id: T-4669
title: "Split crates/rustsheet/src/workbook.rs below the 1000-line limit"
milestone: M16
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
crates/rustsheet/src/workbook.rs has room to grow: it is split into cohesive modules under crates/rustsheet/src/workbook/ and stays well below the 1000-line file-size limit, with no behaviour change.

## Context
T-4151 left workbook.rs at exactly 1000 lines (scripts/check_file_size.py's limit), having moved `recalculate` to workbook/calculation.rs to make room for the pivot load-time refresh. The next change to the file would fail the ratchet. Candidates to move: the load path (`open`, `from_xlsx_bytes`, the import error mapping) into workbook/load.rs, and the reads (`value`, `values`, `cell`, `formatted`, `used_range`, `stored_cells`) into workbook/read.rs. The unit tests at the bottom of the file can move with what they test. crates/rustsheet/src/edit.rs is at 994 lines after T-4152 too: move its pivot and table-edit plumbing into edit/ modules in the same pass, or file a sibling task.

## Acceptance criteria
- [x] workbook.rs is under 700 lines; no public item changes path or signature.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
workbook.rs is 333 lines (was 1000). The load path (`open`, `from_xlsx_bytes`,
`pin_clock_from_metadata`) moved to workbook/load.rs, the saves (`to_xlsx_bytes`, `save_xlsx`) to
workbook/save.rs, the reads (`top_left`, `cell_at`, `text_utf16`, `value`, `values`, `cell`,
`formatted`, `used_range`, `stored_cells`) to workbook/read.rs, and the unit tests to
workbook/tests.rs. All of it moved verbatim; only the imports changed. No public item changed path or
signature. The edit.rs split is filed as T-4828.
