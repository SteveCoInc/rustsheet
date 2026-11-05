---
id: T-4258
title: "Collation: where a leading iteration or long-vowel mark sorts against the plane-2 ideographs (M2 exit cells t3465-collation-holdout C313, D313)"
milestone: M2
lane: engine
status: done
priority: P0
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Excel's `<`, `>` and `=` order a text that starts with a long-vowel or iteration mark (`ー ゝ ゞ ヽ ヾ 々`, with no
letter before it) against the astral ideographs as Excel does. The two M2 exit cells of probes/t3465-collation-holdout
pass, and the rule rests on probe evidence, not on that single pair.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main 3c8d8c7d3). These two cells are the only cells that keep the M2 exit
  from being met. In the report's "Milestone exit" table, M2 probes reads `2 [2] not met`. They are the only failing
  cells of the M0-M2 probe row.
  - probes/t3465-collation-holdout.case.json (2): Sheet1!C313, D313
- Evidence (excel-graph, 2026-11-03, T-3465): A313 is `UNICHAR(131072)` (U+20000 `𠀀`). B313 is
  `UNICHAR(12293)&UNICHAR(863)&UNICHAR(12530)&UNICHAR(811)&UNICHAR(848)&UNICHAR(12446)`, i.e. `々` U+3005, U+035F,
  `ヲ`, U+032B, U+0350, `ゞ`.
  - C313 `=A313<B313`: Excel TRUE, ours FALSE.
  - D313 `=B313<A313`: Excel FALSE, ours TRUE.
  - E313 `=A313=B313` is FALSE on both sides.
- Ours: `LONE_MARK` in crates/rustsheet-engine/src/functions/util/collation.rs is a mark with no letter before it. It
  weighs one step after the last BMP unified ideograph (`OTHER_BASE + 2 * (IDEOGRAPHS + 0x9FFF) + 1`). The doc
  comment cites `"ー" > "人人"` (probes/t3275-collation-width-kana-pairs AC25, AC37).
  - coercion.md, "Other scripts, ... and astral characters (T-3276)", puts plane 2 (`𠀀`, `𪜀`) above the BMP
    ideographs (ranks 152 < 153 < 154). So the engine puts the lone mark between `一` and `𠀀`.
  - Excel puts it above `𠀀`. Where it sits against `𪜀` (U+2A700), plane 3 (`𰀀` U+30000, which ranks *below* Hangul),
    the CJK compatibility ideographs and Extension A is unprobed.
  - coercion.md's hold-out paragraph (T-3465) names this pair as T-3381's to settle. T-3381 is a P2 sonnet research
    probe over many other kana choices. This task takes the two exit cells and the rule they need. T-3381 keeps the
    rest of its list.
- Probe first (excel-probe skill, a new `probes` case; probes-numeric is frozen to decision 0267 tasks). Suggested
  pairs, each as `<`, `>` and `=`:
  - each lone mark `ー ゝ ゞ ヽ ヾ 々` against `𠀀`, `𪜀`, `𰀀`, `가`, `一`, `鿿` (U+9FFF), `㐀` (U+3400), `豈` (U+F900),
    a private-use character (U+E000), `😀` and U+FFFD;
  - `々` followed by a combining mark (`々`&U+035F), by kana (`々ヲ`) and by an ideograph, against `𠀀`, to show whether
    anything after the leading mark changes the first-level comparison;
  - the B313 text against `𠀀` again, as a control.
- Semantics: coercion.md "Long-vowel and iteration marks after a kana or kanji repeat it" ("Alone or first ... they
  sort after the ideographs probed"). Update that paragraph and the `LONE_MARK` doc comment with the probed place.

## Visible-parity stage
Target: probes/t3465-collation-holdout Sheet1!C313 and D313 give Excel's booleans, with no other cell of
t3465-collation-holdout, t3275-collation-width-kana, t3275-collation-width-kana-pairs, text-collation-order or
t3559-collation-unprobed-blocks regressing.

## Acceptance criteria
- [x] A new oracle-filled probe case (excel-graph) covers the pairs above. coercion.md records the lone marks' place
      against the astral and compatibility ideographs as VERIFIED and cites its cells.
- [x] The two known-failure lines `F probes/t3465-collation-holdout.case.json Sheet1!C313` and `Sheet1!D313` disappear
      from eval/baseline/probes.txt with no new ones (`make eval-baseline`). The M2 row of the report's "Milestone exit"
      table reads `0 [0] met` for probes.
- [x] Every cell of the new probe case passes, or each one that fails is a known failure that a task filed in this PR
      owns.
- [x] A unit test in collation.rs pins `"𠀀" < "々"` and the other probed boundaries.
- [ ] `make check-fast` green locally, CI green; reviewers approve.

## Out of scope
- T-3381's other choices: voicing against the Latin marks, full-width against superscript, `ー` after `ん`, after an
  ideograph or after a digit, the katakana-only kana, wildcard folds and the half-width `ｰ`.
- Marks after a Latin letter (the non-transitive `"aー"` rule, T-3321).

## Outcome
- Probe: probes/t4258-collation-lone-mark-astral (105 cells, excel-graph 2026-11-05). Each lone mark
  `ー ゝ ゞ ヽ ヾ 々` is greater than every character it was set against: plane 2 (U+20000, U+2A700, U+2FA1D), plane 3
  (U+30000), Hangul, the BMP ideographs, Extension A, the compatibility ideographs, private use (U+E000, U+F0000),
  U+1F600, U+1D400, U+FFFD and U+10FFFD (C1:C90). It is also greater than two-character high texts (C104:C105). What
  follows the leading mark does not change that (C91:C103, including t3465's B313 text as a control).
- Engine: `LONE_MARK` (collation.rs) moves from one step above U+9FFF to one weight above every script weight
  (`scripts::ABOVE_SCRIPTS`). `IDEOGRAPHS` is private to scripts.rs again. A new unit test,
  `lone_mark_sorts_above_every_character` in test_collation.rs, pins the probed boundaries.
- coercion.md: the lone-mark paragraph records the probed place as VERIFIED, and the T-3465 hold-out note points at
  the fix.
- Eval delta: probes/t3465-collation-holdout C313 and D313 now pass, and the new case's 105 cells all pass. This
  meets the M2 probes exit row.
