---
id: T-4013
title: "= , COUNTIF and XMATCH(..,0) equate T-3584's MATCH classes and keep supplementary case pairs apart (t3584 B21, B25, P7, P9; t3929 rows 29-30)"
milestone: M2
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's text comparison (`=`, the COUNTIF-family text criteria without a wildcard, and XMATCH's exact mode) treats the
members of T-3584's equivalence classes as equal, and the supplementary case pairs as different, and the engine does
the same: `UNICHAR(7941)=UNICHAR(7946)` (U+1F05 with U+1F0A), `ϵ=ε`, U+1FBE `=ι`, `Ͷ` with `ϳ` and `ϑ` with `ϴ` are
equal, and Deseret U+10400 with U+10428 is not. This is the half of T-3904 that does not wait on T-3662's
weightless-position probe.

## Context
- Evidence (excel-graph, 2026-11-03): coercion.md "Weightless characters in `=`, COUNTIF and XMATCH (T-3584)",
  its bullet "Not about weightless characters"; lookup.md "Equivalence classes"; criteria.md "Case beyond ASCII"
  (T-3929: COUNTIF, SUMIF, COUNTIFS and XMATCH(..,0) without a wildcard equate `Ͷ`/`ϳ` and `ϑ`/`ϴ`).
- Owned cells (split from T-3904, which no longer lists them):
  - probes/t3584-match-fold-weightless (8): Calc!B21, D21, F21, B25, D25, F25, P7, P9
  - probes/t3929-wildcard-fold-criteria (6): Calc!B29, C29, D29, B30, C30, D30
  - probes/t3929-wildcard-fold-xmatch (2): Calc!E29, E30
- Code: Excel's `=` and the text criteria go through `cmp_text_pairwise`, the sorts and XMATCH's exact index through
  `cmp_text` and `CollationKey` (crates/rustsheet-engine/src/functions/util/collation.rs; texts equal under the
  pairwise test must keep equal keys). T-3445's `CLASSES` table and `fold_lookup_char`
  (functions/util/collation/lookup_fold.rs) already hold these classes for exact MATCH and the wildcard paths:
  reuse that one table (directive 3), do not copy it.
- Open design point, for the implementer to settle with the evidence above: whether the classes enter the collation
  as one primary weight per class or, for the Greek Extended letters, as the accent-weight sums of T-3154
  (coercion.md marks that UNVERIFIED). Either way the order of class members under `<` is unprobed: keep it
  consistent with equality (`<` FALSE both ways for equal texts) and record it as UNVERIFIED in coercion.md, or probe
  it in the same PR with a few `<` cells if the choice changes any existing collation-order cell.
- Deseret: every supplementary character has a weight (lookup.md), and Excel keeps the case pair apart under `=`
  (Calc!B25 FALSE), so the comparison must not case-fold astral letters.

## Acceptance criteria
- [ ] The 16 cells above pass and leave eval/baseline/probes.txt; `make eval-check` shows 0 regressions in every
      suite (in particular probes/t3465-collation-holdout, probes/text-collation-order, probes/t3211-collation-sort-cycles
      and probes/t3181-collation-prefix-callers keep every cell they pass today).
- [ ] The class table is one definition shared with `fold_lookup_char` (no second copy of `CLASSES`).
- [ ] Unit tests: one pair per class kind (Greek Extended, `ϵ`/`ε`, U+1FBE/`ι`, `Ͷ`/`ϳ`, `ϑ`/`ϴ`) is equal under
      `cmp_text_pairwise` with equal `CollationKey`s, and U+10400/U+10428 is not.
- [ ] coercion.md's T-3584 section says "Implemented in" with this task for the class and Deseret bullet, and states
      how `<` orders class members (VERIFIED with cells, or UNVERIFIED).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The weightless characters (U+0524, U+00AD, Mtavruli, Cherokee small letters, and Calc!P39's U+037F): T-3904, after
  T-3662.
- Exact MATCH, VLOOKUP and HLOOKUP and the wildcard paths (already done by T-3445 and T-3929).
