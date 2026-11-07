---
id: T-5010
title: "Stage 5: keep VBA in a regenerated macro-enabled save, with its content type and codeName (SPEC-0034 D5.1, decision 0275 D5)"
milestone: M15
lane: engine
status: blocked
priority: P2
size: M
tier: opus
depends_on: [T-5000, T-5003]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
A regenerated save of a macro-enabled source writes a macro-enabled package that keeps its `vbaProject.bin` (and
`vbaProjectSignature`) byte for byte. It uses the macro-enabled main content type, and keeps the `codeName` attributes
of `workbookPr` and every `sheetPr`.

## Context
- SPEC-0034 D5.1's `Active` row, and decision 0275 D5: VBA is carried byte for byte into macro-enabled files.
- The export writes no macro-enabled package today: no `macroEnabled` content type and no `vbaProject` handling in
  crates/rustsheet-xlsx/src/export. A regenerated save of an `.xlsm` therefore loses its macros.
- The Verbatim and Patched tiers already keep the package as it was.
- A non-macro output drops the parts with `ActiveContentInPlainFormat`, which SPEC-0033 D7.2 scores as a pass.

## Acceptance criteria
- [ ] The output's main content type follows the source's: `sheet.macroEnabled.main+xml`, and the template and add-in
      forms.
- [ ] `vbaProject` and its signature are kept, with their relationship.
- [ ] `codeName` is kept on `workbookPr` (T-5003) and on `sheetPr` (T-5001).
- [ ] A fallback save of an `.xlsm` fixture keeps its macros (fixture test).
- [ ] Saving to a plain `.xlsx` drops them, and that is reported.
- [ ] An Excel for the web re-save probe (T-4890's `--save-dir`, `rustsheet-oracle resave`, `package-diff`) shows the saves open and keep what this task keeps. Findings go into docs/excel-semantics/xlsx-storage.md (D5.8). VBA retention also gets a desktop Excel check (M14) before its entry is VERIFIED (D5.7).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the output format the caller asked for.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275): M15 waits until Phase 1 parity exits. Unblock when
docs/status.md has the line `**Phase 1:** exited <date>`. An owner-directed session may claim it earlier
(decision 0306).
