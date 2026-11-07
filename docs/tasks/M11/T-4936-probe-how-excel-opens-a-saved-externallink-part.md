---
id: T-4936
title: "Probe how Excel opens a saved externalLink part: a fixture, an empty-target book and rich-error cache cells"
milestone: M11
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4143]
---

## Goal
Oracle evidence on whether Excel opens the externalLink parts our export writes (T-4143) without a repair prompt,
and reads their cached values back.

## Context
T-4143 (PR #3958) writes one externalLink part per external book. docs/excel-semantics/xlsx-storage.md "Links to
other workbooks are an externalLink part each, saved back as read" marks Excel's reading of the saved parts as
UNVERIFIED. There are three open points (the T-4143 fidelity review's MINOR findings):
- A saved fixture: load `eval/suites/fixtures/cases/external-link-cached-lookups.xlsx`, save it with
  `save_xlsx_to_writer`, have the oracle open it and recalculate, and compare the `[1]` cells with the source
  oracle values.
- A book with no target (a DDE or OLE link, read as an empty book) is saved as an empty `externalBook` with
  `Target=""`. Does Excel open that, and do the later `[n]` keep their books?
- Cached errors go through `exported_error_literal`, so `#SPILL!`, `#CALC!`, `#BLOCKED!`, `#CONNECT!` and
  `#FIELD!` become `#VALUE!`. Does Excel ever write those kinds in an externalLink cache, and what does it
  read back?
Use the `excel-probe` skill. Record the findings in the xlsx-storage.md entry, and file engine tasks for any
mismatch.

## Acceptance criteria
- [ ] Probe cases (or a documented reason one cannot be built) for each of the three points, with oracle values committed.
- [ ] The xlsx-storage.md entry's status is updated with the evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Refreshing links (no I/O, directive 5).
