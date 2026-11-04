---
id: T-4060
title: "Probe three collation choices T-3904/T-4039 left open: space forms at the first pass, the prefix rule under SORT and MATCH(..,1), marks after a weightless character"
milestone: M2
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An excel-graph probe case settles three collation choices that T-3904 and T-4039 (PR #3107) made without evidence.
docs/excel-semantics/coercion.md "Format characters, U+200B and the C1 controls (T-3662)" then states each one as
VERIFIED, or an engine task is filed for the cells that differ.

## Context
- From the excel-fidelity review of #3107 (three MINORs). The engine's choices:
  1. **Space forms at the first pass.** U+202F (`NARROW_NBSP`) and U+2000 to U+200A (`SPACE_FORMS`) have primary
     weights of their own, while U+3000 is the space with a full-width variant. probes/t4039-collation-space-forms
     ranks single characters only, so it cannot tell a primary weight from a later-pass one.
     Probe: `x&"a"<" b"`, `x&"a"<CHAR(160)&"b"`, `" "&"b">x&"a"` for each x.
  2. **The prefix rule under sorts.** `cmp_prefix` applies only in `cmp_text_pairwise` (`=`, `<`, the criteria). The
     total order (SORT, SORTBY, MATCH(..,1), VLOOKUP(..,TRUE)) and `CollationKey` treat `"ab"` and `"ab"&U+00AD`
     as equal, and `""` as equal to `➿`.
     Probe: SORT and MATCH(..,1) over `ab`, `ab`&U+00AD, U+00AD&`ab`, `AB`.
  3. **Marks after a weightless character.** `elements` drops a weightless character before `decompose` and
     `resolve`, so a combining mark after it attaches to the letter before it (`"a"&U+00AD&U+0301` weighs as `"á"`).
     Probe: `"a"&UNICHAR(173)&UNICHAR(769)="á"`, `<"á"`, `>"á"` and the same with U+0524.
- Method: the `excel-probe` skill, milestone M2, under about 150 cells.

## Acceptance criteria
- [x] A probe case `probes/t4060-collation-open-choices` with its excel-graph sidecar answers 1 to 3.
- [x] coercion.md records each answer with cells (VERIFIED). Every cell the engine fails is a known failure owned by an
      engine task filed in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (the engine task this may file).

## Outcome
- Probe probes/t4060-collation-open-choices (222 cells, excel-graph, 2026-11-04). Findings in coercion.md, T-3662
  section, "T-4060 settled three choices":
  1. U+3000, U+205F and U+202F share the space's first-pass weight. CHAR(160), CHAR(9), CHAR(10), CHAR(13) and
     U+2000 to U+200B have their own, and U+2000 to U+200A sort above CHAR(10) and CHAR(13), which resolves T-4039's
     UNVERIFIED note. The engine is wrong only for U+202F.
  2. SORTBY, MATCH(..,1), VLOOKUP(..,TRUE), XMATCH(..,±1) and UNIQUE apply the prefix rule (Excel's pairwise
     comparison; SORTBY's tie order depends on the input order). The engine already passes every cell, because
     those paths use `cmp_text_pairwise`. GROUPBY and PIVOTBY are left UNVERIFIED.
  3. A combining mark after a weightless character is a lone mark: `"a"&U+00AD&U+0301` sits between `"a"` and
     `"á"`. The engine attaches it to the `a`.
- New engine task T-4072 owns the 9 off cells (Calc!B2, K2, B50, B51, B55, B56, B58, B59, B63).
- Eval delta: fixed 0, regressions 0; 1 new case, 213 of 222 pass.
