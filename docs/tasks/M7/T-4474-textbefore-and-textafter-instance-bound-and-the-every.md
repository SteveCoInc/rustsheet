---
id: T-4474
title: "TEXTBEFORE and TEXTAFTER instance bound and the every-delimiter-too-long check in UTF-16 units: probe a multi-byte text or delimiter and a match_mode 1 delimiter whose full fold is longer, and match"
milestone: M7
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-4473]
---

## Goal
TEXTBEFORE and TEXTAFTER decide "every delimiter is longer than the text" and the instance_num bound in the units
Excel uses, so a text or delimiter outside ASCII, and a match_mode 1 delimiter whose full case fold is longer than
itself, give Excel's `#N/A`, `#VALUE!` or result. The rule is probed first, then implemented, and text.md's
"Unprobed" sentence on it is replaced by evidence.

## Context
- docs/excel-semantics/text.md "TEXTBEFORE and TEXTAFTER: instance_num past the text, a number if_not_found"
  (probes/t2314-textbefore-instance-bound, probes/t2581-textsplit-textbefore A30-A50, T-2581) evidences the bound in
  UTF-16 units and ends: "Unprobed: a delimiter whose case folding changes its UTF-16 length under match_mode 1 (ours
  measures the unfolded delimiter), and a delimiter that fits the text by UTF-8 bytes but not by UTF-16 positions (A50
  is decided by ours' earlier every-delimiter-longer check)".
- text.md "TEXTSPLIT, TEXTBEFORE, TEXTAFTER and REGEX* lift ..." (probes/textsplit-lift-guesses A10-A21, T-1824): when
  every entry is longer than the text the call is `#VALUE!` with more than two arguments and `#N/A` with two.
- Ours: `Model::text_around_element` (crates/rustsheet-engine/src/functions/text/before_after.rs) tests
  `exact.iter().all(|d| d.len() > text.len())`, which compares UTF-8 bytes, so `TEXTBEFORE("a","é",1)` (1 byte against
  2, but one UTF-16 unit each) is `#VALUE!` here where a UTF-16 count would reach the ordinary `#N/A`; and a text that
  is longer in bytes only (`TEXTBEFORE("é","ab")`, `TEXTBEFORE("😀","abc")`) skips that check and falls to the
  instance bound, which is `#VALUE!` whatever the argument count, where the two-argument rule says `#N/A`. The bound
  (`shortest`, `positions`) counts the unfolded delimiter in UTF-16 units.
- Write one probe case, probes/t4474-textbefore-utf16-bounds (excel-probe skill, excel-graph, tag M7), each result as
  its text or error, every row once with two arguments and once with instance_num 1 (and TEXTAFTER beside TEXTBEFORE):
  - bytes against units: `TEXTBEFORE("a","é")`, `TEXTBEFORE("é","ab")`, `TEXTBEFORE("😀","abc")`,
    `TEXTBEFORE("😀","ab")`, `TEXTBEFORE("ab","😀")`, `TEXTBEFORE("a",{"é","bb"})`, `TEXTAFTER("a","é",1,0,0,"nf")`;
  - instance bound: `TEXTBEFORE("éé","x",3)` and `4`, `TEXTBEFORE("😀a","x",3)` and `4`;
  - folded length under match_mode 1, with match_mode 0 controls: `TEXTBEFORE("ab","İ",2,1)` and `3`,
    `TEXTBEFORE("ab","ß",2,1)` and `3`, `TEXTBEFORE("a","ß",1,1)` and `TEXTBEFORE("a","İ",1,1)` (`İ` folds to two
    units and `ß` to `ss` under full folding, one unit each under ours);
  - TEXTSPLIT controls: `TEXTSPLIT("a","é")` and `TEXTSPLIT("é","ab")`.

## Acceptance criteria
- [ ] probes/t4474-textbefore-utf16-bounds is committed with oracle values (excel-graph) for every formula above.
- [ ] Every cell of it passes (`make eval-show CASE=probes/t4474-textbefore-utf16-bounds`); the too-long check and the
      instance bound measure the text and the delimiters with one helper in before_after.rs (no second length rule),
      commented with the probe cells.
- [ ] probes/t2314-textbefore-instance-bound, probes/t2581-textsplit-textbefore, probes/textsplit-lift-guesses and
      probes/textbefore-casefold-length still pass; the eval gate shows 0 regressions in every suite.
- [ ] docs/excel-semantics/text.md: the "Unprobed: a delimiter whose case folding ..." sentence is replaced by the
      probed rules and cells.
- [ ] A unit test pins `TEXTBEFORE("a","é",1)` and `TEXTBEFORE("é","ab")` at their probed values.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which characters match under match_mode 1 (the fold itself): T-4473, which this follows because both touch the
  match_mode 1 path.
- TEXTSPLIT's own splitting rules and argument kinds (T-2876); the empty delimiter (text.md "TEXTBEFORE and TEXTAFTER
  with the empty delimiter", already evidenced).
- PIVOTBY subtotal work (T-4388, T-4401, T-4414, T-4428), function arrays (T-4449) and eta names (T-4450) are unrelated.
