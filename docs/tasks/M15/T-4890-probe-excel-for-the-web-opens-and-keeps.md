---
id: T-4890
title: "Probe: Excel for the web opens and keeps what the patch tier saves (repair check)"
milestone: M15
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: [T-4514, T-4561]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
Evidence from Excel itself that the source-preserving export's saves open cleanly and keep what we wrote. The test
is that Excel for the web opens each patched save and saves it again, and what its save drops or rewrites is
measured. That evidence moves the UNVERIFIED rows of docs/excel-semantics/xlsx-storage.md ("A patched package
keeps the source's hints") to VERIFIED, or to a fix.

## Context
- T-4560, T-4561 and T-4623 patch the source package instead of regenerating it. Some hints stay as the source had
  them:
  - row `spans`;
  - a `<dimension>` that only grows;
  - rows emptied by clearing cells;
  - a kept or dropped calcChain;
  - added metadata parts.
- No oracle run has opened such a package yet. T-4180 is the full golden flow; this is the narrower probe it builds
  on.
- The excel-graph oracle already uploads a workbook, edits it, and downloads Excel's save
  (`GraphBackend.recalc`, tools/src/rustsheet_tools/oracle/backends.py).

## Acceptance criteria
- [ ] `rustsheet-eval roundtrip --save-dir <dir>` writes each case's R0 and R1 outputs.
- [ ] `rustsheet-oracle resave` uploads each to Excel for the web and stores Excel's save. A file Excel refuses is
      recorded as refused.
- [ ] `rustsheet-eval package-diff <ours> <excel>` scores our save against Excel's re-save with the round-trip
      differ.
- [ ] The probe runs over the fixtures' R0 and R1 outputs and a corpus sample, and the findings go into
      docs/excel-semantics/xlsx-storage.md with evidence paths.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel (M14).
- Scoring the suites against Excel's goldens (T-4180).
