---
id: T-4935
title: "externalLink export hardening: plain attribute escaping, linear sheet dedup, non-finite cached numbers"
milestone: M11
lane: engine
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-4143]
---

## Goal
The externalLink export (T-4143) writes link targets, sheet names and defined names exactly as read. A part with
many `sheetData` elements loads in linear time. A cached number that is not finite is never written as `NaN` or
`inf`.

## Context
These are the MINOR findings of the T-4143 review (PR #3958, security and design reviewers):
- `crates/rustsheet-xlsx/src/export/external_links.rs` escapes the rels `Target`, `sheetName@val` and
  `definedName@name`/`refersTo` with `escape_xml`. That function also applies the shared-string `_xHHHH_`
  convention, which the rels and sheetName readers do not decode, so a target or sheet name holding `_x0041_`
  saves as `_x005F_x0041_`. Before adding a plain XML attribute escaper, search `export/escape.rs` and the rest
  of the export (`rg "fn escape"`) and reuse or extend what exists. Check which readers decode `_xHHHH_` in
  attributes before deciding per attribute.
- `crates/rustsheet-xlsx/src/import/external_links.rs` dedups `cached_sheets` and `refresh_error_sheets` with
  `Vec::contains` once per `sheetData`. That is O(n²) in a count the untrusted file controls. The export calls
  `refresh_error_sheets.contains` once per sheet. Use a set, or sort and dedup.
- `cell_value` writes `format!("{n}")`, so a non-finite number gives `NaN`/`inf`. Excel rejects those in `<v>`.
  Find out how the cell export handles one (`saved_number`, worksheets.rs) and do the same.
- `sheet_data_set` skips a cell whose column `number_to_column` cannot spell after its `<row>` is open. This is
  only reachable from an API-built book. Comment on it, or skip the cell before opening the row.

## Acceptance criteria
- [ ] A round trip keeps a target and a sheet name that contain `_x0041_` (unit or integration test).
- [ ] The importer's dedup is not quadratic: a test with many distinct `sheetData` sheetIds stays fast, or the code is plainly linear.
- [ ] A non-finite cached number is written as the cell export writes one (test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- How Excel opens a saved part (T-4936).
