---
id: T-4074
title: "Probe the x87 decimal read in typed input, percent text, criteria, NUMBERVALUE and FORMULATEXT"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3932]
---

## Goal
Oracle evidence says whether the readers that share `parse_excel_number` with literals and VALUE read short decimals through Excel's x87 multiply by 10^-n, and how FORMULATEXT prints such a literal.

## Context
- numbers.md "Decimals reach a double through an x87 multiply by 10^-n": T-3932 evidenced literals and VALUE
  (probes-numeric/t3658-round-scaling, -fresh). The same reader, `parse_excel_number` in
  crates/rustsheet-engine/src/number_format.rs, also serves typed cell input (`parse_formatted.rs`, percent
  text included), `"text"+0`, criteria operands, NUMBERVALUE, AVERAGE's text and DATEDIF's number text; those
  take the conversion unprobed (UNVERIFIED in that entry; T-3932 fidelity review).
- `literal_text` in stringify/literals.rs prints `=4910E-9` as `4910e-9` and `=14550E-15` as `14550e-15` so a
  reparse keeps the value; Excel's FORMULATEXT of those literals is unknown.
- Points: model-off decimals such as 4.91E-06 (491 × 10^-8, one ulp above the nearest double) and 14550E-15,
  with a nearest-double control (4910E-9). Read the bits with `*2^k-c` companions as the t3658 cases do.

## Acceptance criteria
- [ ] A probes-numeric case, oracle-filled (excel-graph): typed input `4.91E-06` and `4.91%`-style percent text
      at a model-off point, `"4.91E-06"+0`, `COUNTIF` with a `"=4.91E-06"` criterion over both doubles,
      `NUMBERVALUE("4.91E-06")`, and `FORMULATEXT` of `=4910E-9`, `=14550E-15`, `=4.91E-06`; plus the decimal-point literal forms
      `=0.00000491`, `=4.91E-06` and `=-0.00000491` (the t3658 cases hold only `mE-n` forms).
- [ ] numbers.md records each result (VERIFIED or the rule found); a task for any engine change is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (file a follow-up).
