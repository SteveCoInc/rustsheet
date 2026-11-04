---
id: T-4240
title: "Tighten the T-4228 range-group record: label format, the <start item, the rounding model, and an owner for the relabel quirk"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-4228]
---

## Goal
docs/excel-semantics/web.md "Grouped pivot fields import as their group items" (numeric-range part) and T-4235's context
claim only what probes/t4228-pivot-range-groups shows.

## Context
MINOR findings of the Excel-fidelity review of T-4228 (PR #3219), all wording:
- "Excel's own labels for decimal data are `start-end` of each step (`0-3`, `0.3-0.4`), formatted General" goes past the
  evidence: the probe's decimal labels already equal what Excel would write, so only pivot 3's relabel (`0-3`, `3-6`,
  `6-9`) shows Excel's own labels. Limit the claim to pivot 3 and mark decimal labels and the General format
  UNVERIFIED (T-4236 can probe them with wrong-text labels of the right count).
- "the `<start` item's bound (-0.5)": -0.5 is a value inside the `<0` group, not a bound. Say "a number inside the
  `<start` group (-0.5) is `#REF!`", and in T-4235 drop "never the `<start` item" (one point, Calc C38).
- T-4235 says the 15-digit rounding is "fixed by pivot 5". Pivots 1 and 5 rule out an absolute tolerance on q, but a
  relative tolerance of about 3.3e-15, or rounding v as well as q, fits too. T-4235 should treat the rule as a
  structural model (reviewer-excel-fidelity check 7) and test the near-boundary points.
- No task owns the relabel-and-reorder quirk for a `<groupItems>` list of the wrong length (pivot 3). Add it to T-4235's
  context (the importer keeps the file's labels), or file its own task.

## Acceptance criteria
- [ ] web.md and T-4235 are reworded as above. No claim goes past the probe's cells.
- [ ] The relabel quirk has an owner task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New probes (T-4236).
