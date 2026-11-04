---
id: T-4156
title: "Probe the typed script-digit shapes T-3891 infers: 11-digit and decimal long numbers, yearless and month-year dates, percent fractions, non-$ currency"
milestone: M5
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for the typed script-digit shapes whose number format T-3891 infers, and the engine matching it.

## Context
T-3891 (PR #3145) gives typed script-digit text Excel's locale-tagged format from probes/t3891-typed-script-digit-shapes
(docs/excel-semantics/coercion.md "Script digits in number text"). `script_digit_format` (formatter/parse_formatted.rs)
infers these unprobed shapes (reviewer-excel-fidelity, PR #3145): an 11-digit integer `١٢٣٤٥٦٧٨٩٠١` and a 12-digit number
with decimals (the `value.abs() >= 1e11` arm), long decimals `١.٢٣٤٥٦٧٨٩٠١٢٣`, a date with no year `١/٢` and `١ Jan`
(ASCII `d-mmm`, ours `[$-,200]dd-mmm-yyyy`), month-year `Jan ٢٠٢٠`, a mixed fraction with `%` `١ ١/٢%` (ours tagged
`0.00%`), a non-`$` currency `€١٢` or `١٢€` (ours a tagged copy of the ASCII format), and a small number `٠.٠٠٠٠٠١`.
Write one excel-probe case (typed text in A, ISNUMBER in B, the value in C as in t3891-typed-script-digit-shapes),
fill it with excel-graph, commit the ENTERED log under docs/research/, record the rules, and fix any arm the log contradicts.
The yearless dates' values move with the clock: score only B.

## Acceptance criteria
- [ ] The probe case is filled; its ENTERED log is committed; coercion.md lists each shape's format as VERIFIED.
- [ ] `script_digit_format` gives the logged format for every probed shape (unit tests in
      formatter/test/test_parse_formatted_number.rs `typed_script_digits_take_tagged_formats`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The `,N00` suffixes beyond those probed (T-4107); typed `$(12)` (T-4117).
