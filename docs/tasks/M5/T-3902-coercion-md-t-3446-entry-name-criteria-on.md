---
id: T-3902
title: "coercion.md T-3446 entry: name criteria on the five new digit blocks as unprobed and fix the row-102 wording (review follow-up)"
milestone: M5
lane: research
status: in-progress
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/coercion.md "Script digits in number text (T-3446)" and T-3891 say exactly what
probes/t3446-script-digits-forms shows, no more.

## Context
Excel-fidelity review of PR #2921 (T-3446), two MINOR findings:
- The entry dropped the T-3276 paragraph's UNPROBED list. Criteria (COUNTIF and the other readers through
  functions/criteria.rs, which share `cast::number_text_in`) reading the five new blocks (Extended
  Arabic-Indic, Bengali, Lao, Tibetan, Myanmar) and script-digit clock and mixed-fraction text are
  inferred from the shared path, not probed: T-3325's COUNTIF cells cover only the three original blocks,
  plain. The entry should say so in one UNPROBED line, or a COUNTIF `"="&C` column should be probed.
- Row 102 of the case (`١.٥`) repeats row 14; it is not a mixed-script row. The entry and the T-3891
  Context call B100:B102 "mixed scripts": only rows 100 and 101 are.

## Acceptance criteria
- [ ] coercion.md's T-3446 entry lists the unprobed criteria forms (or a probe column with oracle values
      settles them and the entry records the result).
- [ ] The row-102 wording is corrected in coercion.md and T-3891.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed input (T-3891).
