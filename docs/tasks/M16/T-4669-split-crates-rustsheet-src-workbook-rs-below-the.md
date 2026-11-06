---
id: T-4669
title: "Split crates/rustsheet/src/workbook.rs below the 1000-line limit"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
crates/rustsheet/src/workbook.rs has room to grow: it is split into cohesive modules under crates/rustsheet/src/workbook/ and stays well below the 1000-line file-size limit, with no behaviour change.

## Context
T-4151 left workbook.rs at exactly 1000 lines (scripts/check_file_size.py's limit), having moved `recalculate` to workbook/calculation.rs to make room for the pivot load-time refresh. The next change to the file would fail the ratchet. Candidates to move: the load path (`open`, `from_xlsx_bytes`, the import error mapping) into workbook/load.rs, and the reads (`value`, `values`, `cell`, `formatted`, `used_range`, `stored_cells`) into workbook/read.rs. The unit tests at the bottom of the file can move with what they test.

## Acceptance criteria
- [ ] workbook.rs is under 700 lines; no public item changes path or signature.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
