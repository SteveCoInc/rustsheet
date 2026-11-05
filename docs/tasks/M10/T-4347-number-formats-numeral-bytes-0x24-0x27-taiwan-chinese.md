---
id: T-4347
title: "Number formats: numeral bytes 0x24-0x27 (Taiwan Chinese, traditional formal, byte-29 variant, Korean hangul)"
milestone: M10
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
`[$-24000409]` to `[$-27000409]` (and the `,2400` to `,2700` suffixes) print Excel's numerals: Chinese with ０ for zero, traditional formal, a byte-29 variant, and Korean hangul.

## Context
T-4107 found numeral bytes 0x20-0x27 are CJK systems, not ASCII, and implemented 0x20-0x23. T-4107 removed its 0x24-0x27 cells (the visible-parity lock refuses a new case with off cells): re-add them as a probe case (e.g. `[$-24000409]General` on 10, 105, 12345, 100000, 1230, `0` on 1230, General on 0.05, and 1010 under 0x24; values recorded in docs/excel-semantics/number-formats.md "Bytes 32-39"). Code: `Numerals::from_byte` (formatter/numerals.rs). Probe date fields and elapsed tokens first (excel-probe); Korean needs its own digit and unit tables.

## Acceptance criteria
- [ ] The cells above pass and a date/clock/elapsed probe for each byte is evidenced.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- Probe probes/t4347-numeral-bytes-24-27 (new, 140 cells, excel-graph 2026-11-05; generator
  docs/research/t4347/gen_probe.py). For each of bytes 0x24-0x27 it covers General on 16 values, `0`,
  `#,##0.00`, percent, scientific, a fraction, date fields (`yyyy-mm-dd yy d`, `m d dd`), clock fields (`h:mm:ss`,
  `hh:mm`), elapsed counts (`[h]:mm`, `[s]` up to 10^12 and past 10^16, padded `[hh]`), and the `,2400` to `,2700`
  suffix. The engine passes all 140.
- `Numerals::from_byte` (formatter/numerals.rs):
  - 36 → `CHINESE_FULLWIDTH_ZERO`: byte 27's structure with a one before every unit and ０ for zero;
  - 37 → `FORMAL_TRADITIONAL_DAIJI`: byte 28's structure with 零壹貳參;
  - 38 → `FULLWIDTH`: byte 29, identical on every probed cell;
  - 39 → `KOREAN`: hangul, with 조 for 10^12.
- Quirk recorded in docs/excel-semantics/number-formats.md ("Bytes 32-39"): Excel writes 六, 零, 參 and 拾 under bytes
  0x24 and 0x25 as CJK compatibility ideographs (U+F9D1, U+F9B2, U+F96B, U+F973).
- Unit test: `bytes_36_to_39_spell_as_excel`.
- Eval delta: probes known failures 4792 -> 4792, 0 regressions; the new case t4347-numeral-bytes-24-27 passes all 140 cells (0 visibly off).

