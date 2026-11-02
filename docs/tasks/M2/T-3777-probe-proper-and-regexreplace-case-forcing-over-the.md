---
id: T-3777
title: "Probe PROPER and REGEXREPLACE case forcing over the old UPPER/LOWER case table characters"
milestone: M2
lane: functions-text
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PROPER and REGEXREPLACE `\U \L \u \l` are evidenced for the characters T-3298 swept, so the engine's choice to follow the UPPER/LOWER case table there (T-3583) is either confirmed or corrected.

## Context
T-3583 moved `simple_case` (used by PROPER) and `upper_case`/`lower_case` (used by REGEXREPLACE case forcing) onto Excel's old case table, which T-3298 evidenced only for UPPER and LOWER. PROPER (`PROPER("άλφα")` is now Αλφα) and REGEXREPLACE on those characters are inferred (text.md, UNVERIFIED). Code: functions/text/string_format.rs, regex_replacement.rs.

## Acceptance criteria
- [x] A probe has PROPER over a word-initial character of each unmapped block plus ά έ ή ί ό ύ ώ and Georgian, and REGEXREPLACE(x,".+","\U$0") / "\L$0" over the t3298 Data strings; oracle values recorded.
- [x] text.md marks the PROPER and REGEXREPLACE entries VERIFIED, and the engine is corrected where Excel differs.
- [x] `make check` green; reviewers approve.

## Out of scope
- MATCH fold (T-3584).

## Outcome
Two probes, filled on excel-graph (2026-11-02): `probes/t3777-proper-case-table` (M5; 162 cells: PROPER over every
t3298 UPPER-swept character word-initial, every LOWER-swept character inside a word and word-initial, Asomtavruli,
spot checks) and `probes/t3777-regexreplace-case-table` (M7; 121 cells: `\U$0` / `\L$0` over the t3298 Data strings,
`\u` `\l` spot checks). Findings, recorded in text.md:
- REGEXREPLACE `\U` `\L` follow the UPPER/LOWER table exactly (confirmed, no change).
- PROPER inside a word follows LOWER's table exactly. At a word start it follows UPPER's table except that the Greek
  tonos vowels keep the tonos (`PROPER("άλφα")` is Άλφα) and Asomtavruli U+10A0-U+10C5 lowers to Mkhedruli: new
  `proper_initial` (string_format.rs).
- `\u` and `\l` change only the next character and end a `\U`/`\L` mode (`\l$0` with ΩA is ωA, `\L$0\uCD` is abCD);
  the engine had `\l` set the `\L` mode (T-1897's lower-case-only evidence could not tell). Fixed in `CaseState::apply`
  (regex_replacement.rs), unit tests added.
Eval delta: +2 probe cases, 283 cells, all bit-exact (13 PROPER and 10 REGEXREPLACE cells fail on main); no
regressions; known failures unchanged elsewhere (probes 3925 -> 3925).
