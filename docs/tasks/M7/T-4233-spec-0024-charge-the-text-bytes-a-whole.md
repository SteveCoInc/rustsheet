---
id: T-4233
title: "SPEC-0024: charge the text bytes a whole-array reader compares (lookups, MATCH, UNIQUE, SORT, the *IF(S)) over a bound array of long texts"
milestone: M7
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A LAMBDA that repeats a text-comparing reader over a LET-bound array of long texts stays within the formula work budget:
the bytes the reader compares are charged, not only the cells it copies (SPEC-0024 section 1, "text values").

## Context
T-2805 charges each read of a bound array by its cells (`evaluate_node_with_reference`, `variable_reference`), but
not by the bytes of its text elements. Those bytes are shared since T-2806, so the copy is cheap, and charging every read
by length would refuse `IF(SEQUENCE(1024,1024),REPT("a",32767))`, which computes today. A reader that compares texts
byte by byte does more work than the copy, though. For example, in
`LET(s,IF(SEQUENCE(1E5),REPT("a",32767)),MAP(SEQUENCE(900),LAMBDA(i,MATCH(REPT("a",32766)&"b",s,0))))`, each call compares
up to 32 KB per element, which is about 3 GB a call within the cell charge. Find the comparing readers (MATCH/XMATCH/XLOOKUP/LOOKUP and
the V/HLOOKUP searches, UNIQUE, SORT/SORTBY, the *IF(S) criteria matching, GROUPBY keys) and charge the bytes they
compare, at a shared comparison chokepoint if one exists (directive 3). See docs/excel-semantics/arrays.md "One formula
evaluation does at most 268,435,456 work units".

## Acceptance criteria
- [ ] The MATCH formula above, and one formula per other comparing reader, is `#VALUE!` (`TOO_MUCH_WORK`) under the
      default limit (engine tests that count units, not time).
- [ ] No baseline gains a line; the DEVIATION entry lists the text-byte charge.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cell charges for array reads (T-2805, done).

## Outcome
- One chokepoint: `functions/util/collation.rs` counts what each text comparison reads (`counted` wraps the passes'
  iterators, so a comparison pays up to its first difference, not the texts' lengths) and what each collation key or
  lookup fold is built from (`note_compared_bytes`); the ASCII arms that bypass it (`ExactKey::text_is` of MATCH,
  VLOOKUP and HLOOKUP's exact match, `folded_units`, `WildcardPattern::matches`) note theirs too. `Model::spend_compared`
  (area_budget.rs) charges the count after each node (`Model::charged`, which `evaluate_node_in_context` ends with), and
  `with_area_budget` keeps an outer run's pending count apart from a dependency cell's run (and, at the top level,
  ticks noted outside any run).
- Calibration (round 2, after fidelity's round-1 finding that one unit per 4 bytes refused single SORTs that compute):
  the count is in ticks weighted by path, as the paths' cost a character differs a hundredfold (collation/compared.rs:
  `BYTE_TICKS` 1, `ELEMENT_TICKS` 32, `CHAR_TICKS` 128, `KEY_TICKS` 128, `FOLD_TICKS` 32; round 3 halved `KEY_TICKS` from 256, which refused
  `ROWS(UNIQUE(REPT("a",20)&SEQUENCE(1048576)))`), and
  `COMPARED_TICKS_PER_UNIT` = 16 (area_budget.rs). Timed on a release build (figures in arrays.md): every path's
  bound-array worst case reaches the limit in 4.9-7.4 s; `ROWS(SORT(REPT("a",100)&SEQUENCE(1048576)))` computes (4.1 s,
  81M compare units). Single nodes whose comparisons take longer than the limit's worth still run to their end before
  they are `#VALUE!` (follow-ups T-4278, stop inside the node; T-4279, speed up the collation).
- `a_sort_of_texts_with_a_long_shared_prefix_stays_well_inside_the_limit`: 65,536-row SORTs (100-character shared
  prefix; equal 30-character texts) spend under a twentieth of the limit;
  `a_unique_of_medium_texts_stays_inside_the_limit`: a 65,536-row UNIQUE of 27-byte texts under a sixteenth.
- Tests (test_formula_work.rs, units not time): `every_text_comparing_reader_pays_the_bytes_it_compares` (MATCH exact,
  wildcard and approximate, XMATCH, XLOOKUP, LOOKUP, VLOOKUP, HLOOKUP, UNIQUE, SORT, SORTBY, GROUPBY, `=`),
  `a_comparison_pays_for_what_it_reads_not_for_the_texts_lengths`, and
  `a_lambda_repeating_a_text_comparing_reader_is_bounded` (the task's MATCH shape and XMATCH over 100 texts under a
  2^24 limit; over the task's 1E5 texts the first MATCH call alone notes about 8.2E8 units, past 2^28, so the MAP is
  `#VALUE!` at the default limit).
- The *IF(S) functions are not readers of a bound array: they take ranges (`COUNTIF(s,..)` over a LET-bound array is an
  error before any comparison), and a criterion over 255 characters is `#VALUE!` first; their comparisons go through
  the same count.
- arrays.md "One formula evaluation does at most 268,435,456 work units" (DEVIATION) lists the charge.
- Eval delta: none (no baseline line changes; no eval formula past the limit).
