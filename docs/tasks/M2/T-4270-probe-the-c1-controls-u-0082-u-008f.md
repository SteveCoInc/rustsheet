---
id: T-4270
title: "Probe the C1 controls U+0082-U+008F and U+0091-U+009D in the collation and weigh them as Excel does (code-point order is an engine guess)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Every C1 control (U+0080 to U+009F) collates in `<`, `=`, SORT, MATCH/XMATCH and COUNTIF where Excel puts it, backed by
excel-graph cells for each of the 32 code points, not only the five probed so far.

## Context
- docs/excel-semantics/coercion.md "Format characters, U+200B and the C1 controls (T-3662)": U+0080, U+0081, U+0090,
  U+009E and U+009F rank 18 to 22 above U+200B and below `0` (probes/t4039-collation-space-forms B19:B23, J19:J23,
  K19:L23), and U+0081, U+0085 and U+009F sit above the punctuation probed (probes/t3662-collation-ignorables-prefix
  B11:L13, M11:N13, P11:P13; probes/t3662-collation-ignorables-sweep K, L, M). The paragraph ends "UNVERIFIED:
  U+0082 to U+008F and U+0091 to U+009D, taken to keep code-point order". 25 of the 32 code points rest on that guess.
- No open task probes them: T-3971 (Math Operators, Hebrew finals, `™`, reordered blocks), T-3203 (accent marks),
  T-3180 (`İ`), T-3381 (kana and width) and T-3305 (accent weights) cover other characters (checked with
  `rg '0082|U\+0091|009D' docs/tasks`).
- Code: `collation.rs` gives every C1 control a weight of its own, `'\u{80}'..='\u{9f}' => single(C1_BASE + (c - 0x80))`
  (crates/rustsheet-engine/src/functions/util/collation.rs, the `C1_BASE` doc comment near `DIGIT_BASE`). If Excel ties
  some of them, or orders them otherwise, this arm and its doc comment change; the weightless class of T-3904 and the
  U+200B weight of T-4039 stay as they are.
- Layout to reuse (`excel-probe` skill): t4039-collation-space-forms (Data!A one `=UNICHAR(n)` per row, Data!B
  `="a"&UNICHAR(n)&"b"`; Calc!B `=SUMPRODUCT(--(Data!A$1:A$n<Data!A1))` the rank, Calc!C the same with `=` the equal
  count, Calc!D:E the same over Data!B), with all 32 code points, plus pairs against U+200B, `•`, `¬` and `0` for the
  ends of the run and an XMATCH/COUNTIF column so the lookups are covered too.
  New probe case name: `t4270-collation-c1-controls`.

## Acceptance criteria
- [ ] probes/t4270-collation-c1-controls is filled on excel-graph (rank, equal count and `a&c&b<ab` for each of
      U+0080..U+009F, and the neighbour pairs above) and every cell passes, with no new F line in eval/baseline/probes.txt
      and no other probes cell regressing (`make eval`).
- [ ] coercion.md replaces the UNVERIFIED sentence with the probed rule (VERIFIED, with the case and cells), and its
      "Implemented in" names the collation arm.
- [ ] A unit test in crates/rustsheet-engine/src/test/test_collation.rs pins the order of the 32 controls.
- [ ] `make check` green; reviewers approve.

## Out of scope
- C0 controls, U+200B, the Unicode spaces and the weightless class (T-3662, T-3904, T-4039, T-4060 settled them).
- Lone surrogates (probes/text-lone-surrogates).
