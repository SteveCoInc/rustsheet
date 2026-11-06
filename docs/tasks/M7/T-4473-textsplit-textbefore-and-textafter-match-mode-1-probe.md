---
id: T-4473
title: "TEXTSPLIT, TEXTBEFORE and TEXTAFTER match_mode 1: probe and match the folds still UNVERIFIED (İ against i and I, final sigma in TEXTSPLIT, Å U+212B, ſ, µ, titlecase digraphs, Σ before a mark)"
milestone: M7
lane: functions-text
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
TEXTSPLIT, TEXTBEFORE and TEXTAFTER with match_mode 1 fold every character text.md still marks UNVERIFIED as Excel
does, so a delimiter matches (or not) exactly where Excel's does. The rules are probed first, then implemented in
the shared fold, and the UNVERIFIED notes in docs/excel-semantics/text.md are replaced by evidence.

## Context
- docs/excel-semantics/text.md "TEXTSPLIT match_mode 1 case folding" (probes/textsplit-casefold-length A1-A8, T-1794)
  leaves UNVERIFIED: the delimiter `İ` (U+0130) against a plain `i` or `I` and `I` against `İ`; final sigma in TEXTSPLIT
  (ours follows SEARCH: `ς` and `Σ` both fold to `σ`); and "the folding of other chars (Å U+212B, ſ, µ, final sigma,
  titlecase digraphs) is UNVERIFIED and follows SEARCH's".
- text.md "TEXTBEFORE and TEXTAFTER match_mode 1 case folding" (probes/textbefore-casefold-length, T-1796;
  probes/textbefore-sigma-contexts, T-1799) leaves UNVERIFIED whether a `Σ` before a combining mark (U+0301), `’`
  (U+2019) or a zero-width space (U+200B) is `σ`; the U+212B, ſ, µ and digraph folds are untested there too.
- Ours: `fold_match_char`, `fold_for_split` and `fold_for_before_after` (crates/rustsheet-engine/src/functions/text/
  casefold.rs) on SEARCH's one-to-one `fold_char` (functions/util/collation.rs, shared with lookups and criteria, so
  it must not change here: an exception goes in `fold_match_char`, as U+0130 and U+212A already do); final sigma is
  `sigma_is_final` (text/string_format.rs, shared with LOWER).
- Excel already departs from SEARCH twice in this fold (U+0130 matches `i`, the Kelvin sign does not match `k`), so
  the other compatibility characters may depart too; nothing may be assumed.
- Write one probe case, probes/t4473-textsplit-fold-gaps (excel-probe skill, excel-graph, tag M7), each TEXTSPLIT read
  through `ARRAYTOTEXT(..)` and `COLUMNS(..)`, each TEXTBEFORE/TEXTAFTER as its text or error, with match_mode 0
  controls beside the first row of each group:
  - İ: `TEXTSPLIT("aibIc","İ",,,1)`, `TEXTSPLIT("aİbİc","I",,,1)`, `TEXTBEFORE("aIb","İ",1,1)`, `TEXTBEFORE("aİb","I",1,1)`;
  - sigma in TEXTSPLIT: the text `"aΣbςcσd"` split by `"σ"`, `"ς"` and `"Σ"`, and `"aΟΣ"` and `"aΣb"` split by `"ς"`;
  - U+212B (Angstrom sign) against `å` and `Å` both ways, U+017F (ſ) against `s` and `S` both ways, U+00B5 (µ)
    against U+03BC (μ) and U+039C (Μ) both ways, U+01C5 (ǅ) against U+01C4 (Ǆ) and U+01C6 (ǆ) both ways, U+03C2 vs
    U+03C3 in TEXTSPLIT both ways; each once in TEXTSPLIT and once in TEXTBEFORE (build the characters with UNICHAR);
  - Σ before a mark: `TEXTBEFORE("ΟΣ"&UNICHAR(769)&"x","σ",1,1)` and the same with `"ς"`, and with UNICHAR(8217) and
    UNICHAR(8203) in place of UNICHAR(769).

## Acceptance criteria
- [x] probes/t4473-textsplit-fold-gaps is committed with oracle values (excel-graph) for every formula above.
- [x] Every cell of it passes (`make eval-show CASE=probes/t4473-textsplit-fold-gaps`); each departure from SEARCH is
      one arm of `fold_match_char` (or of `fold_for_before_after` for a TEXTBEFORE/TEXTAFTER-only rule) with a comment
      citing its probe cells; `fold_char` and `sigma_is_final` are unchanged unless SEARCH's and LOWER's own probes
      (probes/text-search-casefold-length, probes/t3820-lower-sigma-next-char) still pass.
- [x] probes/textsplit-casefold-length, probes/textbefore-casefold-length and probes/textbefore-sigma-contexts still
      pass; the eval gate shows 0 regressions in every suite.
- [x] docs/excel-semantics/text.md: both case-folding entries state the probed rules with their cells and drop the
      UNVERIFIED notes they answer.
- [x] A unit test pins one TEXTSPLIT and one TEXTBEFORE probed departure (or the agreement) next to the T-1794/T-1796 tests.
- [x] `make check` green; reviewers approve.

## Out of scope
- The instance_num bound and the too-long check measured in UTF-16 units, including a delimiter whose full fold is
  longer: T-4474.
- SEARCH, FIND, EXACT, lookup and criteria folds (`fold_char` callers): no change here.
- TEXTSPLIT/TEXTBEFORE argument kinds (empty text, errors, ranges): T-2876. PIVOTBY subtotal work (T-4388, T-4401,
  T-4414, T-4428), function arrays (T-4449) and eta names (T-4450) are unrelated.

## Outcome
- probes/t4473-textsplit-fold-gaps (117 cells, excel-graph, 2026-11-06) answers every UNVERIFIED fold in the task: 93 cells
  passed on main, 24 did not; all 117 pass now.
- Two departures from SEARCH's fold, both now in casefold.rs:
  - The Angstrom sign U+212B matches neither `å` nor `Å` in TEXTSPLIT, TEXTBEFORE or TEXTAFTER (A27:A38). It is an arm of
    `fold_match_char`, beside the Kelvin sign.
  - TEXTSPLIT folds sigma by position like TEXTBEFORE (A14:A23, A75:A80): `ς` and `σ` never match, a `Σ` before a letter
    is `σ`, a final or lone `Σ` is `ς`. The loop moved out of `fold_for_before_after` into `fold_sigma_by_position`,
    which both folds call; `fold_char` and `sigma_is_final` are unchanged.
- Agreements with the existing code, now evidenced: `İ` against `i`/`I` both ways (A3:A11), `ſ` and `µ` match no other
  form, the titlecase digraphs match their upper and lower forms (A39:A74, A81:A104), and a `Σ` before U+0301, U+2019 or
  U+200B is `σ` (A105:A117).
- docs/excel-semantics/text.md: both case-folding entries state these rules and drop the UNVERIFIED notes they answer.
  Still UNVERIFIED: `ẞ` against `ß` in TEXTSPLIT, and whether a letter must precede the `Σ`.
- Unit tests: `test_textsplit_case_insensitive_sigma_angstrom_and_digraph`,
  `test_textbefore_case_insensitive_dotted_i_angstrom_and_sigma_before_mark`.
- Eval delta: 1 new case, 117/117 cells passing; 0 fixed and 0 regressions in every other suite.
