---
id: T-4118
title: "Extended-hex tag [$-_409] (0x2F409) keeps English day halves: h A/P is \"6 PM\", not \"6 P\" (t1924 S!B22)"
milestone: M10
lane: functions-text
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(0.75,"[$-_409]h A/P")` is "6 PM", as in Excel: the extended-hex tag `[$-_409]` keeps the en-US day halves
of its English LCID instead of falling to "no locale" and printing the `A/P` letters as written.

## Context
- Gap triage 2026-11-04 (`make eval` on main 4556dbc0, excel-graph). `make exit-owners MILESTONE=M10` and
  `make unowned-off ALL=1` list this as the only M10 exit cell with no open owner. T-3666 (done) took it over from
  T-1875 but its Outcome fixed only the plain hex `[$-F409]`; the cell still fails and the F line is still in
  eval/baseline/probes.txt.
- Cells owned:
  - probes/t1924-locale-tag-extended-hex.case.json (1): S!B22
- Excel vs ours (`make eval-show CASE=probes/t1924-locale-tag-extended-hex.case.json`):
  `=TEXT(0.75,"[$-_409]h A/P")` expected "6 PM", actual "6 P" (TextMismatch). Neighbouring evidence:
  probes/text-locale-invalid-sublanguage prints "6 PM|6 PM|6 PM" for `[$-F409]h AM/PM|h A/P|h a/p` (English is
  exempt from the undefined-sublanguage rule), while `[$-x_y]` (0x2412) prints English names with the letters kept
  (text-locale-calendars A139).
- Root cause: `extended_hex` reads `_409` as 47*16^3 + 0x409 = 0x2F409, i.e. calendar digit 2 (Gregorian, en-US
  names) over LCID 0xF409. `resolve_extended` (crates/rustsheet-engine/src/formatter/locale_tag.rs) sends every LCID
  above 0x3FF with no table of its own to `without_locale`, which returns `LocaleTag::Silent` for digit 2, so `A/P`
  stays as letters. The plain-hex path (`resolve_lcid` -> `by_language`) treats 0xF409 as English, which is what
  Excel does here too. Do not simply route `resolve_extended` through `resolve_lcid`: `[$-fr_FR]` (0x10E00B, LCID
  0xE00B, Finnish language with an unbounded sublanguage) would then pick up Finnish names and break
  text-locale-calendars A135-A145. The narrow evidenced rule is: an extended-hex LCID whose language is English
  (`lcid & 0x3FF == 0x09`) resolves as `resolve_lcid` does. Other English sublanguages reached through extended hex
  are unprobed; say so in the doc comment.
- docs/excel-semantics/number-formats.md (the extended-hex paragraph under the locale-tag rule) calls `[$-_409]`
  "0xF409" and says "0xF409 follows English (T-3666)". Correct it to 0x2F409 (digit 2 over 0xF409) and name this
  task as the one that implements it. Exotic locale tag: no `campaign:` key (decision 0114).

## Acceptance criteria
- [ ] The F line `probes/t1924-locale-tag-extended-hex.case.json S!B22` disappears from eval/baseline/probes.txt with
      no new F or TF lines in any suite (`make eval-baseline` refuses regressions); text-locale-calendars
      A135-A145, text-locale-invalid-sublanguage and the other 59 t1924 cells still pass.
- [ ] A unit test in formatter covers `[$-_409]h A/P` ("6 PM") and `[$-x_y]h A/P` (letters kept, "6 P").
- [ ] number-formats.md gives 0x2F409 and cites S!B22; the doc comment on `resolve_extended` states the English
      exception and that other English sublanguages via extended hex are unprobed.
- [ ] `make exit-owners MILESTONE=M10` reports 0 unowned cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Probing other extended-hex tags over English sublanguages (file a research task if the doc comment's unprobed
  note matters for a failing cell).
- The lowercase en-IE/en-BZ/en-HK halves (T-4066).
