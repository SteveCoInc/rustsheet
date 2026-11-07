---
id: T-5043
title: "Probe built-in number format ids 23-26 and 82-163 on plain cells"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4731]
---

## Goal
Every built-in number format id below 164 that the engine resolves is backed by Excel evidence of what Excel en-US shows for it and saves it as.

## Context
T-4731 (probes/t4731-builtin-num-fmt-ids, excel-graph 2026-11-07) probed ids 14-17, 22, 27-36 and 50-81 on plain cells: Excel en-US shows each East Asian id as an en-US built-in and saves it as that id (number-formats.md "Built-in ids Excel en-US maps to other built-ins"). Still unprobed: the reserved ids 23-26 and the ids 82-163, which `DEFAULT_NUM_FMTS` (crates/rustsheet-engine/src/number_format.rs) resolves to General. Excel might show some as General, map them to another built-in, or refuse the file. Also unprobed: whether a typed `m/d/yyyy h:mm` value is saved as 22, and the 1904 system's cell display of the date ids. The builder writes a bare built-in id from an integer `number_formats` value (spec2xlsx, T-4731).

## Acceptance criteria
- [ ] A probe of plain cells in ids 23-26 and a sample of 82-163 (with 43831.75, 1.25, -1234.5), filled with excel-graph and its download committed (decision 0342), records Excel's displayed text and saved ids; the engine matches it or each off cell has an owner.
- [ ] The probe also has a typed `1/2/2020 10:00` cell: its saved id shows whether Excel saves `m/d/yyyy h:mm` as built-in 22, which the engine's `get_default_num_fmt_id` now gives (T-4731).
- [ ] A copy of the probe in a 1904 workbook (`"date_system": 1904`) records the displayed text of the date cells, including negatives (dates.md "Cell display of a negative under a date format"; probed only in 1900 by T-4731).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Locales other than en-US.
