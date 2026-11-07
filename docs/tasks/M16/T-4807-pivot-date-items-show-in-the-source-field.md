---
id: T-4807
title: "Pivot date items show in the source field's date number format"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A pivot over a field of dates shows its date items in the source cells' date number format, as Excel does.

## Context
Found by T-2085: on probes/t2085-getpivotdata-items sheet D, rows When over date serials formatted m/d/yyyy,
Excel shows the items as 1/1/2024, 1/2/2024 and 2/1/2024, while the engine shows 45292, 45293 and 45323. The values
match; only the shown text differs. See pivots.md "Item casing when cache and source disagree, and date items" for
what is known about date items.

Owner of these visibly-off cells (decision 0245):
- probes/t2085-getpivotdata-items D!A4:A6

## Acceptance criteria
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] pivots.md "Item casing when cache and source disagree, and date items" says the engine now matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GETPIVOTDATA with date items (T-4805).

## Handoff
Tried (sonnet run): read the render path. Date labels render in `item_cell` (crates/rustsheet-engine/src/pivot/layout/render.rs:125) as
`PivotCell::Number`; `RenderedCell.number_format` is a `numFmtId` applied by `set_pivot_number_format` (pivot/grid.rs ~700), so
label cells could already carry a format. The blocker: `CacheField` (pivot.rs:104) keeps no source number format, and
`refresh.rs::read_source` / `is_date_style` (~line 105-130) only tests date-ness through `SourceCells::value`.
Plan: add `number_format: Option<u32>` to `CacheField` (OOXML `cacheField numFmtId`), fill it in `read_source` from the first date
source cell's xf `num_fmt_id` (extend `SourceCells`), render date-item labels with it via `put_formatted`, and read it on
xlsx import. About 25 `CacheField { .. }` construction sites need the new field (`..Default` where possible). Persisted model plus
trait change, so judged beyond sonnet. Nothing implemented; `make eval-show` was not completed.
