---
id: T-5042
title: "Probe which id Excel gives a new custom number format when the file's custom ids have a gap or start above 164"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4731]
---

## Goal
The engine numbers a new custom number format as Excel does when it saves, for workbooks whose custom ids are not 164, 165, ...: backed by a committed Excel-saved file.

## Context
T-4731 (probes/t4731-builtin-num-fmt-ids, committed download eval/roundtrip/excel-saved/cases/t4731-builtin-num-fmt-ids.xlsx) showed that Excel for the web keeps a file's custom ids 164 and 165 and gives the two formats it adds the next ids, 166 and 167, in entry order. The engine's `get_new_num_fmt_index` (crates/rustsheet-engine/src/number_format.rs) takes the first free id from 164, which agrees there but not necessarily when the file's ids have a gap (164 and 170: first free 165, or highest plus one 171?) or start above 164 (200 only), nor whether Excel renumbers the file's own ids on save. The builder (XlsxWriter) always numbers custom formats from 164, so the probe needs spec2xlsx to write chosen custom ids (a styles.xml patch, like the other spec patches), with a Rust adapter mirror kept in step. Typed Arabic-Indic numbers (`١.٢٥`) make Excel add custom `[$-ar-SA,200]` formats (number-formats.md "Custom number format ids on save"). UNVERIFIED there.

## Acceptance criteria
- [ ] A probe whose workbook carries custom ids with a gap (e.g. 164 and 170) and one starting above 164, plus typed entries that make Excel add formats, is filled with excel-graph and its download committed under eval/roundtrip/excel-saved/cases/ (decision 0342); number-formats.md records the ids Excel kept and gave.
- [ ] `get_new_num_fmt_index` matches Excel's rule, test first, or the difference has an owner.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locales other than en-US; desktop Excel (excel-graph only).
