---
id: T-4060
title: "Probe three collation choices T-3904/T-4039 left open: space forms at the first pass, the prefix rule under SORT and MATCH(..,1), marks after a weightless character"
milestone: M2
lane: research
status: in-progress
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
- [ ] A probe case `probes/t4060-collation-open-choices` with its excel-graph sidecar answers 1 to 3.
- [ ] coercion.md records each answer with cells (VERIFIED). Every cell the engine fails is a known failure owned by an
      engine task filed in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing collation.rs (the engine task this may file).
