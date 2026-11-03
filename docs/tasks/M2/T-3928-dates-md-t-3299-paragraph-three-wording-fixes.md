---
id: T-3928
title: "dates.md T-3299 paragraph: three wording fixes from the review of #2969 (row 7 criteria, slash row order, 2026 run as context)"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3299]
---

## Goal
docs/excel-semantics/dates.md's T-3299 paragraph (probes/t3299-date-text-glued-month-3) and its research
note say exactly what the probe's cells show.

## Context
- The reviewers of PR #2969 (T-3299) approved it with three MINOR findings, left for this task so the
  approved head needed no new push:
  1. The intro says "E, F = 0 and K = 0 wherever DATEVALUE parses". Row 7 (`"15Jul5"`, 2005-07-15)
     parses, but Data!A1:A8 holds no 2005-07-15, so J7 = 0 and K7 = -38548. Say K = 0 where DATEVALUE
     parses and Data holds that date (every parsed row but 7).
  2. The slash bullet lists `"1/Jul2024"`, `"1/July2024"`, `"1/jul2024"`, `"1/Jul24"` as "(rows 10, 13,
     14, 17)", but row 14 is `"1/Jul24"` and row 17 is `"1/jul2024"`. Pair the strings with their rows.
  3. docs/research/t3299-glued-month-typed-input-2026-11-03.md describes the first (2026) run's formats in
     prose with no raw lines and no committed sidecar. Mark that sentence as unscored context; the
     evidence is the raw 2024 ENTERED log.
- Docs only: no case, sidecar or engine change.

## Acceptance criteria
- [ ] The three passages are reworded as above, each citing its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New probes; the engine gap (T-3919).
