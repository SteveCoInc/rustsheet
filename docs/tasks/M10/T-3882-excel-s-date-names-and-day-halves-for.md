---
id: T-3882
title: "Excel's date names and day halves for LCIDs 0x21, 0x24, 0x27, 0x28, 0x2F and 0x3F (id, sl, lt, tg, mk, kk): probe the tables and add them (8 t1924 and text-locale-calendars cells)"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A number-format locale tag that resolves to Indonesian (0x21), Slovenian (0x24), Lithuanian (0x27), Tajik (0x28),
Macedonian (0x2F) or Kazakh (0x3F) prints Excel's own month and day names and day halves for that language, not
en-US. Every string comes from a probe, never from CLDR or memory.

## Context
- Gap triage 2026-11-03 (`make eval` on main 08cba82c, excel-graph). `scripts/exit_owners.py --milestone M10` lists
  these 8 cells as the only M10 exit cells no open M10 task owns. They are probe cells, and M10's exit needs its
  probes at 100%.
- Milestone: the catalog puts TEXT at M5, but this task is M10 like the other locale-tag tasks (T-1924, T-1875,
  T-3666). The cells count toward M10's exit row, and only an M10 task owns them there (decision 0096).
- The extended-hex reading of these tags is already implemented (T-1924, `extended_hex` in
  crates/rustsheet-engine/src/formatter/locale_tag.rs). `[$-x]` is 0x21, `[$-{]` 0x24, `[$-~]` 0x27, `[$-1_]` 0x3F
  and `[$-_]` 0x2F. What is missing is the name tables: docs/excel-semantics/number-formats.md (the extended-hex
  paragraph) says "the missing tables (0x21, 0x24, 0x27, 0x28, 0x2F, 0x3F) are T-1875's". T-1875 is done without
  them, so no open task owns them. `LOCALES` in crates/rustsheet-engine/src/locale/excel_date_names.rs has 42
  probed locales and none of these six.
- Excel vs ours (`make eval-show CASE=probes/t1924-locale-tag-extended-hex.case.json`), all of
  `TEXT(45000,"[$-..]mmmm|yyyy-mm-dd|dddd|e|h AM/PM")` unless noted. Ours is "March|2023-03-15|Wednesday|2023|12 AM"
  each time:
  - A1 `[$-28]` "март|2023-03-15|чоршанбе|2023|12 AM". A4 `[$-1_]` "Наурыз|2023-03-15|сәрсенбі|2023|12 AM".
  - A10 `[$-{]` "marec|2023-03-15|sreda|2023|12 dop.". A28 `[$-~]` "kovas|2023-03-15|trečiadienis|2023|12 priešpiet".
  - A29 `[$-x]` "Maret|2023-03-15|Rabu|2023|12 AM".
  - B1 `TEXT(0.75,"[$-28]h A/P")` and B4 `"[$-1_]h A/P"`: Excel "6 PM", ours "6 P". Tajik and Kazakh print `A/P` as
    their full afternoon string, as 400 and 800 do. Slovenian `[$-{]h A/P` is "6 pop." and Lithuanian `[$-~]` "6 popiet"
    (B10, B28, owned by T-3666).
  - text-locale-calendars A140 `TEXT(45000,"[$-_]mmmm|yyyy-mm-dd|dddd|e")`: Excel "март|2023-03-15|среда|2023",
    the Macedonian names.
- The evidence so far is one month (March), one weekday (Wednesday) and the day halves per language. That is not
  enough to fill a table (`mmm`, `mmmm`, `ddd`, `dddd`, AM, PM), so this task needs a probe first (excel-probe skill),
  like probes/text-locale-name-tables and text-locale-regional-lcids (T-1875). Probe each language under its full
  LCID (0x421, 0x424, 0x427, 0x428, 0x42F, 0x43F) and under its BCP-47 tag (id-ID, sl-SI, lt-LT, tg-Cyrl-TJ,
  mk-MK, kk-KZ). Cover all 12 months in `mmm` and `mmmm`, all 7 days in `ddd` and `dddd`, `AM/PM` and `A/P` before
  and after noon, and `mmmmm`. Lithuanian and Slovenian may decline month names in a day-month context
  (`d mmmm`), so probe that too and record what Excel does.
- Split from T-3666 (sonnet): T-3666 also lists t1924 cells that need these same tables (A2 `[$-428]`, A3 `[$-0_]`,
  B2, B3, B10, B28, B29) beside its own 0x2412 language-fallback question. This task does not list them. When the
  tables land, the Outcome records which of T-3666's cells now pass, and T-3666 is updated in the same PR to drop
  them.
- Exotic locale: per the triage rules, no `campaign:` key.
- Cells owned:
  - probes/t1924-locale-tag-extended-hex.case.json (7): S!A1, A4, A10, A28, A29, B1, B4
  - probes/text-locale-calendars.case.json (1): S!A140

## Acceptance criteria
- [ ] A probe case (excel-graph, canary-verified) holds the full name tables and day halves of the six languages
      described under Context. Its values are recorded in docs/excel-semantics/number-formats.md, and the
      "missing tables ... are T-1875's" sentence is updated.
- [ ] The six tables are in crates/rustsheet-engine/src/locale/ (`LOCALES`, or `regional_date_names.rs` beside
      the other probed tables), each string cited to the probe. There are no CLDR-derived strings.
- [ ] The 8 owned cells pass, and so does the new probe case. Their known-failure lines disappear from
      eval/baseline/probes.txt. `make eval-baseline` records no new known-failure line in any
      eval/baseline/<suite>.txt.
- [ ] T-3666's t1924 cells that pass after this change are listed in the Outcome, and T-3666's Context drops them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 0x2412 and 0xF409 language-fallback rule (T-3666). Numeral-system digits (T-1925, T-3532). Calendar digits
  over these LCIDs.
