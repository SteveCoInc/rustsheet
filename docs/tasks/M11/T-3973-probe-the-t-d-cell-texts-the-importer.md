---
id: T-3973
title: "Probe the t=\"d\" cell texts the importer reads as #VALUE! or empty without evidence: past 9999-12-31, 24:00:00, unpadded dates, no <v>, 1900-02-29 in a 1904 workbook"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Each `t="d"` cell form that the importer currently maps to `#VALUE!` or to an empty cell by guess has an Excel for
the web result in the fixtures suite. For each form, xlsx-storage.md then says either that Excel agrees or which engine
task fixes the mismatch.

## Context
- Gap triage 2026-11-03 (opus, `make eval` on origin/main 69b7d87b). Every failing cell has an open owner, so this
  triage turned to the UNVERIFIED engine guesses that no open task names (directive 1: behavior comes from oracle
  evidence). docs/excel-semantics/xlsx-storage.md "t="d" cells" lists as UNVERIFIED: "dates after 9999-12-31,
  `24:00:00`, non-zero-padded dates (the importer maps these to `#VALUE!` too) and a `t="d"` cell with no `<v>`
  (imported empty)". The 1904 entry below it adds "`1900-02-29` (the 1900 system's phantom day) in a 1904 workbook;
  it is before the epoch, so the importer reads it as `#VALUE!` too". T-0409 and T-2515 (both done) probed the
  other forms. T-3182 covers pivot cache `<d>` items, not cells.
- Code: `iso8601_to_serial` and `iso8601_to_date_system_serial` (crates/rustsheet-xlsx/src/import/iso_date.rs).
  The importer turns `None` into `#VALUE!`. Its doc comment says the upper date bound is unprobed.
- Method: copy the T-0409 and T-2515 fixtures (eval/suites/fixtures/README.md: `t-d-pre-1900.xlsx` cut to row 1, A1
  the `t="d"` cell, B1 = `A1`, C1 = `ISNUMBER(A1)`, D1 = `TEXT(A1,"yyyy-mm-dd hh:mm:ss")`). Use one form per
  workbook, because Excel for the web rejects a whole workbook with unparsable `t="d"` text (`unsupportedWorkbook`).
  A rejection is itself the result, so record it. Fill with
  `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite fixtures`.
- Forms, one workbook each (1900 system unless noted):
  `10000-01-01`, `9999-12-31T23:59:59.999`, `9999-12-31T24:00:00`, `1990-01-01T24:00:00`, `24:00:00`,
  `1990-1-1`, `1990-01-1T12:00:00`, `<c t="d"/>` (no `<v>`), `<c t="d"><v></v></c>` (empty `<v>`), and in a
  `date1904="1"` workbook `1900-02-29` and `1904-01-01T24:00:00`.

## Acceptance criteria
- [ ] The workbooks above are committed under eval/suites/fixtures/cases with excel-graph sidecars, or the README
      records that excel-graph refused the workbook (with its error). Each workbook has a README entry in the
      T-0409 style that gives the value Excel shows.
- [ ] docs/excel-semantics/xlsx-storage.md replaces each UNVERIFIED item with what Excel does, citing the fixture
      and date. Each item Excel refuses says so.
- [ ] Each cell where the engine differs from Excel is recorded with `make eval-baseline` and has an owner. Either an
      engine task filed with `make new-task` (lane engine, milestone M11, `campaign: semantics` and a
      `## Visible-parity stage` section) names it, or this PR fixes it with a one-line change in `iso8601_to_serial`
      and a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Pivot cache `<d>` items (T-3182).
- Typed date input (T-3463) and DATEVALUE text (functions-date tasks).
