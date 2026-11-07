---
id: T-5056
title: "Save East Asian built-in number format ids as their en-US ids and write Excel's 5-8 numFmt entries"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4731]
---

## Goal
A workbook the engine saves carries the number format ids and `numFmt` entries Excel en-US writes for the same cells.

## Context
T-4731 (probes/t4731-builtin-num-fmt-ids; its Excel-saved download eval/roundtrip/excel-saved/cases/t4731-builtin-num-fmt-ids.xlsx) showed that Excel en-US rewrites each East Asian built-in id to the en-US built-in it shows (27 -> 14, 32 -> 21, 59 -> 1, ..., 81 -> 47) and writes `numFmt` entries for the built-ins 5-8 it uses, with the symbol quoted (`"$"#,##0_);\("$"#,##0\)` and so on). The engine maps the ids for display only (`en_us_builtin_id` in crates/rustsheet-engine/src/number_format.rs): on save it keeps the file's id (`cell_xf_xml` in crates/rustsheet-xlsx/src/export/styles.rs writes 27 as 27) and writes no entries for 5-8. number-formats.md "Built-in ids Excel en-US maps to other built-ins" (Ours line).

## Acceptance criteria
- [ ] Saving a workbook whose cells use ids 27-36 and 50-81 writes their en-US ids, test first, in both the full save and the patch tier, or each tier's difference has an owner.
- [ ] A saved workbook that uses built-ins 5-8 carries Excel's `numFmt` entries for them, or the entry records why not (decision record if it changes what R0 keeps).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locales other than en-US.
