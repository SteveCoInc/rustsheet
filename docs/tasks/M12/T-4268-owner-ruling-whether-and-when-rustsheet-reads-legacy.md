---
id: T-4268
title: "Owner ruling: whether and when RustSheet reads legacy .xls (BIFF8) workbooks (SPEC-0031 .xls plan)"
milestone: M12
lane: api
status: blocked
priority: P3
size: S
tier: opus
spec: docs/specs/SPEC-0031-encrypted-xlsx-and-legacy-xls.md
depends_on: []
---

## Goal
An owner ruling on whether RustSheet will read legacy binary `.xls` (BIFF8) workbooks, and if so in which milestone,
recorded as a decision.

## Context
- SPEC-0031 "The `.xls` plan": Excel opens `.xls` with no password, so not reading it is a known gap. A BIFF8
  reader is a second importer (record stream, SST, formats, BIFF formula tokens, shared/array formulas, names,
  external links): L+ and several specs. eval/ has no `.xls` fixtures today.
- Until the ruling, T-4264's `LegacyBinaryWorkbook` refusal is the behavior. This task is blocked on the owner.

## Acceptance criteria
- [ ] A decision record holds the owner's ruling: read `.xls` (milestone, spec task filed) or not (scope note in
      xlsx-storage.md and docs/milestones.md).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Any implementation.

## Blocked because
Blocked because: needs an owner scope ruling (SPEC-0031 "The `.xls` plan").
