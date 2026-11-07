---
id: T-4807
title: "Pivot date items show in the source field's date number format"
milestone: M16
lane: engine
status: done
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

T-4132's gen-pivot batch shows the same on every pivot with the date field When on an axis or as a page field: the
date items' cells and texts (`12/31/2022` where the engine shows 44926), and the labels that hold an item, which are
text cells (`2/29/2024 Total` against `45351 Total`, `12/31/1900 Sum of Score` against `366 Sum of Score`), in the
source cells' format (m/d/yyyy, or m/d/yyyy h:mm with a time). Owner of these cells too (320 values and
texts on 50 sheets; a sheet's span is the bounding box of its date cells):
- gen-pivot/01865529b5a09c0b P02!P7:U7
- gen-pivot/01865529b5a09c0b P05!M7:O7
- gen-pivot/01865529b5a09c0b P07!L8:L12
- gen-pivot/01865529b5a09c0b P11!M6:N6
- gen-pivot/01865529b5a09c0b P12!N7:Q7
- gen-pivot/2251b0f5e5a256e1 P01!M6:M13
- gen-pivot/2251b0f5e5a256e1 P03!N6:Q6
- gen-pivot/2251b0f5e5a256e1 P04!O6:W6
- gen-pivot/2251b0f5e5a256e1 P05!N10:N19
- gen-pivot/2251b0f5e5a256e1 P08!L8
- gen-pivot/2251b0f5e5a256e1 P09!L8:L23
- gen-pivot/2251b0f5e5a256e1 P10!N8:N14
- gen-pivot/2251b0f5e5a256e1 P11!M7:AB7
- gen-pivot/36996978427905b2 P02!L9:L45
- gen-pivot/36996978427905b2 P03!L7:L21
- gen-pivot/36996978427905b2 P04!L7:L23
- gen-pivot/36996978427905b2 P06!N11:N21
- gen-pivot/36996978427905b2 P08!M7:AD7
- gen-pivot/462f10551d480f9d P03!N9:N42
- gen-pivot/462f10551d480f9d P04!M8:M79
- gen-pivot/462f10551d480f9d P05!M8:M42
- gen-pivot/462f10551d480f9d P06!O10:O34
- gen-pivot/462f10551d480f9d P07!M7:Q7
- gen-pivot/462f10551d480f9d P09!O7:R7
- gen-pivot/462f10551d480f9d P10!O6:S6
- gen-pivot/462f10551d480f9d P11!L11:L19
- gen-pivot/640780d7b6f4ad53 P04!O6
- gen-pivot/640780d7b6f4ad53 P06!M6:M13
- gen-pivot/640780d7b6f4ad53 P09!N9:N19
- gen-pivot/a6d74990f3ae755e P01!N8:N13
- gen-pivot/a6d74990f3ae755e P03!M8:M31
- gen-pivot/a6d74990f3ae755e P04!N7:AB7
- gen-pivot/ca5c8fa99c13d747 P02!L9:L18
- gen-pivot/ca5c8fa99c13d747 P05!M6:S6
- gen-pivot/ca5c8fa99c13d747 P08!M9:M11
- gen-pivot/ca5c8fa99c13d747 P12!M6:P6
- gen-pivot/ec473260bb2065b3 P01!L9:L35
- gen-pivot/ec473260bb2065b3 P04!N7:N21
- gen-pivot/ec473260bb2065b3 P05!M6:O6
- gen-pivot/ec473260bb2065b3 P07!M9:M13
- gen-pivot/ec473260bb2065b3 P08!M7:V7
- gen-pivot/ec473260bb2065b3 P09!N7:AZ7
- gen-pivot/ec473260bb2065b3 P11!P6:S6
- gen-pivot/ec473260bb2065b3 P12!L8:L16
- gen-pivot/f9e7edbb5b74011a P02!M6:M12
- gen-pivot/f9e7edbb5b74011a P03!O7:V7
- gen-pivot/f9e7edbb5b74011a P06!N10:N27
- gen-pivot/f9e7edbb5b74011a P08!L10:L17
- gen-pivot/f9e7edbb5b74011a P09!L7
- gen-pivot/f9e7edbb5b74011a P12!L8:L23

## Acceptance criteria
- [x] The cells above pass, and no other case regresses (`make eval-baseline`).
- [x] pivots.md "Item casing when cache and source disagree, and date items" says the engine now matches.
- [x] `make check` green; reviewers approve.

## Out of scope
- GETPIVOTDATA with date items (T-4805).

## Outcome
The Goal's premise was not quite Excel's: a date item does not take the source cells' format but a fixed
short date, `m/d/yyyy`, with ` h:mm` when the item has a time part. probes/t4164-pivot-date-times (source
`m/d/yyyy h:mm`) shows midnight as `4/17/2024`, not `4/17/2024 0:00`; t2085 D and t4130-pivot-date-filters
(source `m/d/yyyy`) show `1/1/2024`. So no source format is carried in the cache (the handoff's plan is not
needed): `RenderedCell::number_format` is now a `CellFormat` (a data field's `numFmtId`, or a code), and a
`CacheItem::Date` label cell gets `date_item_format` (pivot/layout/render.rs); the grid writer applies a code
as it applied an id. The probe-case test model now applies each sheet's `number_formats` and typed error
cells, so `date_item_labels_show_as_short_dates` (pivot/grid/tests.rs) refreshes date-formatted sources as
Excel does. pivots.md "Item casing when cache and source disagree, and date items" records the rule and its
UNVERIFIED edges (seconds, time-only items, date subtotal labels, a selected date page item).

Eval delta: probes text failures 754 -> 640 (114 fixed: t2085-getpivotdata-items D!A4:A6, 86 in
t4130-pivot-date-filters, 25 in t4164-pivot-date-times), 0 regressions; value known failures unchanged.

Adopted as an orphan (work-loop step 0.3): merging `origin/main` met T-4957's date group item format
(`numFmtId` 14). `item_format` now gives a grouped field's date item `CellFormat::Id(14)` and any other
date item `date_item_format`; T-4957's ungrouped-date test now expects `m/d/yyyy`. The applied
`number_formats` make a refresh of t4170's date column read date items, so the added-groups test's
expected cache holds `CacheItem::Date` (the CI failure on the pre-merge head). The export now writes a
date format on label cells, so the 7 pivot-export sidecars whose export changed were refilled
(`rustsheet-oracle fill -b excel-graph --force`): Excel shows the exported labels as `1/1/1900` ...
`3/2/1900`. Post-merge eval delta: probes text 115 -> 1 (114 fixed), pivot-export text 139 -> 125 (14
fixed), gen-pivot 258 cells fixed, 0 regressions.
Review round 1 (adoption): pivots.md now records the 1904 serial 0 `12:00:00 AM` exception (VERIFIED for that
cell; T-4943 narrowed to it); xlsx-storage.md updated; the remaining UNVERIFIED edges are T-5049.

## Handoff (resumed; superseded by the Outcome above)
Tried (sonnet run): read the render path. Date labels render in `item_cell` (crates/rustsheet-engine/src/pivot/layout/render.rs:125) as
`PivotCell::Number`; `RenderedCell.number_format` is a `numFmtId` applied by `set_pivot_number_format` (pivot/grid.rs ~700), so
label cells could already carry a format. The blocker: `CacheField` (pivot.rs:104) keeps no source number format, and
`refresh.rs::read_source` / `is_date_style` (~line 105-130) only tests date-ness through `SourceCells::value`.
Plan: add `number_format: Option<u32>` to `CacheField` (OOXML `cacheField numFmtId`), fill it in `read_source` from the first date
source cell's xf `num_fmt_id` (extend `SourceCells`), render date-item labels with it via `put_formatted`, and read it on
xlsx import. About 25 `CacheField { .. }` construction sites need the new field (`..Default` where possible). Persisted model plus
trait change, so judged beyond sonnet. Nothing implemented; `make eval-show` was not completed.
