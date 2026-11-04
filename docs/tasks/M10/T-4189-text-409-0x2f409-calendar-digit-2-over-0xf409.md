---
id: T-4189
title: "TEXT: [$-_409] (0x2F409, calendar digit 2 over 0xF409) keeps en-US day halves, so h A/P is \"6 PM\" (probes/t1924-locale-tag-extended-hex S!B22, the one ownerless M10 exit cell)"
milestone: M10
lane: functions-text
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(0.75,"[$-_409]h A/P")` returns Excel's "6 PM", so the last known failure of
probes/t1924-locale-tag-extended-hex passes and the M10 exit row has no ownerless cell.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main c3d019be8, excel-graph evidence). `make phase1-exit` reports
  "owners: not met ... 1 milestone exit cells with no owner of their milestone (M10 1)". That cell is this one:
  - probes/t1924-locale-tag-extended-hex.case.json (1): S!B22
  - `=TEXT(0.75,"[$-_409]h A/P")`: Excel "6 PM", ours "6 P" (TextMismatch; known failure in eval/baseline/probes.txt
    since T-1924). S!A22 under the same tag already passes ("March|2023-03-15|Wednesday|2023|12 AM").
- Ownership history: T-1875 and then T-3666 named B22. T-3666 (done) fixed plain hex `[$-F409]`
  (probes/text-locale-invalid-sublanguage S!F3 "6 PM|6 PM|6 PM" passes), but B22 still fails. Its Outcome does not
  mention B22, so the cell has had no open owner since then.
- The Excel rule is already recorded. docs/excel-semantics/number-formats.md, "More regional variants" / "Invalid
  sublanguage (T-3666)", says: "English is exempt: `[$-F409]` (and `[$-_409]`) has English names and en-US halves
  '6 PM|6 PM|6 PM'". The "Resolution" bullet records the extended hex reading: `_` counts 47 = 0x2F, so `_409` is
  0x2F409, which is calendar digit 2 (Gregorian, en-US names) over LCID 0xF409.
- Suspected root cause, in crates/rustsheet-engine/src/formatter/locale_tag.rs:
  - `resolve_extended` sends every extended-hex LCID above 0x3FF that has no table of its own to `without_locale`.
    For calendar digit 2 that returns `LocaleTag::Silent`, so `A/P` is kept as written ("P").
  - The plain-hex path (`resolve_lcid`, then `by_language` with `sublanguage_defined`) reaches en-US for 0xF409, and
    that gives "PM".
  - So the English exemption T-3666 added does not reach the extended reading.
  - Keep `[$-x_y]` (0x2412, English names with the letters kept; text-locale-calendars A139) and `[$-fr_FR]`
    (0x10E00B, Saka with en-US names) as they are. Under the plain-hex fallback 0x10E00B would become Finnish, so do
    not route every extended value through `by_language`. Scope the change to English (language 0x09) LCIDs.
- Exotic locale tag: no `campaign:` key (gap-triager rule). No new Excel evidence is needed.

## Acceptance criteria
- [ ] probes/t1924-locale-tag-extended-hex S!B22 passes, and its F line disappears from eval/baseline/probes.txt
      (`make eval-baseline`), with no new known-failure line in any suite.
- [ ] Every other cell of probes/t1924-locale-tag-extended-hex, text-locale-calendars,
      text-locale-invalid-sublanguage and text-locale-regional-lcids-2 keeps its current verdict.
- [ ] A unit test in the formatter covers `[$-_409]h A/P` → "6 PM". It also checks two evidenced contrasts: `[$-10D90B]h A/P`
      → "6 P" (t1924 S!B15) and `[$-x_y]mmmm` of 45000 → "March" (text-locale-calendars A139). `[$-x_y]`'s day
      halves are unprobed, so do not assert them.
- [ ] The doc comment on `resolve_extended` and the number-formats.md "Resolution" bullet say that English
      extended-hex LCIDs follow the T-3666 exemption, citing S!B22.
- [ ] `make check-fast` green; reviewers approve.

## Out of scope
- Other languages' sublanguage bounds under the extended reading (unprobed).
- Lowercase day halves for en-IE, en-BZ and en-HK (T-4066).
