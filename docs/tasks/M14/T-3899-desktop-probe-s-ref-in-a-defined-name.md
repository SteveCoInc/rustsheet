---
id: T-3899
title: "Desktop-probe S!#REF! in a defined-name body after rename, in a duplicated sheet, and a typed name of a missing sheet"
milestone: M14
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Desktop Excel evidence for three shapes of a sheet-qualified `#REF!` that T-3062 implemented without an oracle.

## Context
T-3062 made `S!#REF!` follow a sheet rename and print `#REF!#REF!` once S is missing. Unprobed (excel-graph cannot
duplicate sheets and prints a typed unknown sheet as an external-link path): (1) a defined-name body `S!#REF!`
after S is renamed; (2) duplicating a sheet whose formulas or global names hold `S!#REF!` (the engine now
rewrites the copy's name and gives a global name a sheet-local copy); (3) the FORMULATEXT of a typed
`=NoSuchSheet!#REF!`: excel-graph reads it as an external workbook `NoSuchSheet` and prints the workbook's
folder URL before `[NoSuchSheet]NoSuchSheet` (values in probes/typed-missing-sheet-ref-error-text), while the
engine prints `#REF!#REF!`; desktop text decides what the engine should print for a file with no URL.
See docs/excel-semantics/references.md "A sheet-qualified #REF! is #REF!".

## Acceptance criteria
- [ ] Desktop-Excel values for the three shapes are recorded in a probe and references.md; the engine matches them or the entry says why not.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
