---
id: T-4897
title: "Probe calculated fields beyond T-4505's shapes"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4174]
---

## Goal
Excel-for-the-web evidence for the calculated-field shapes T-4505 did not try, so that the engine's
choices for them (T-4174, marked UNVERIFIED) are confirmed or corrected.

## Context
T-4174 implements calculated fields from pivots.md "Calculated fields as Excel for the web renders them"
(probes/t4505-pivot-calc-fields, t4505-pivot-calc-field-functions). These shapes are engine choices there,
each UNVERIFIED with this task (crates/rustsheet-engine/src/pivot/calculated.rs, result.rs `calculate`):
- a custom subtotal (`item@t`) on a row field beside a calculated data field (the engine shows the formula's value);
- a calculated data field under show-values-as, with a number format, on a grouped field;
- a value filter or Top-N whose `iMeasureFld` is a calculated data field (the engine has no result: Malformed);
- a field name written in another case than the field's (`units*2`; the engine matches ignoring case), and a
  field named like a cell (`Rev2`) named bare in another formula;
- an operand field holding an error value (the engine passes the error into the formula);
- a field that only names a cycle member (`Cz = Ca+1` beside the cycle Ca, Cb; the engine shows `#NAME?`), and
  `IFERROR(Ca,0)` over a cycle member;
- the unprobed refusals the engine applies at import: RANDBETWEEN, RANDARRAY, OFFSET, CELL, INFO, GETPIVOTDATA,
  IMAGE, STOCKHISTORY, FIELDVALUE, FILTERXML, LET, LAMBDA, a range (`Units:Price`), a call of an unknown function;
  and a formula past 255 or 8,192 characters;
- a formula whose result is an array (`SEQUENCE(2)`) or empty;
- a formula whose result is a bare boolean (`Units>3`, `TRUE`; the engine shows 1 or 0, as AND's result shows);
- a calculated field beside Sum-only totals and beside a Count total over an error field (the engine reads a
  calculated data field as Sum for both rules);
- a refresh after the source loses a field a formula names, or gains a column named like a calculated field.

## Acceptance criteria
- [ ] Probe cases with sidecars for the shapes above that the web opens; pivots.md entries updated, refusals
      added to "Pivot flags Excel for the web refuses".
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (follow-up tasks if the evidence differs).
- Calculated items (T-4792, T-4175).
