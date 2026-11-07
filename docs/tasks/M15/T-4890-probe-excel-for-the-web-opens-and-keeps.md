---
id: T-4890
title: "Probe: Excel for the web opens and keeps what the patch tier saves (repair check)"
milestone: M15
lane: harness
status: done
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
- [x] `rustsheet-eval roundtrip --save-dir <dir>` writes each case's R0 and R1 outputs.
- [x] `rustsheet-oracle resave` uploads each to Excel for the web and stores Excel's save. A file Excel refuses is
      recorded as refused.
- [x] `rustsheet-eval package-diff <ours> <excel>` scores our save against Excel's re-save with the round-trip
      differ.
- [x] The probe runs over the fixtures' R0 and R1 outputs and a corpus sample, and the findings go into
      docs/excel-semantics/xlsx-storage.md with evidence paths.
- [x] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel (M14).
- Scoring the suites against Excel's goldens (T-4180).

## Outcome
- **Tools.**
  - `rustsheet-eval roundtrip --save-dir DIR` writes `<case>.<scenario>.xlsx` for every scenario run.
  - `rustsheet-eval package-diff OURS THEIRS [--json]` scores THEIRS against OURS with the round-trip differ. Its
    semantic side is OURS as stored, not recalculated (`scenarios::load_stored`), so a file compared with itself
    reports nothing (`a_package_diff_of_a_workbook_against_itself_finds_nothing`).
  - `rustsheet-oracle resave -b BACKEND DIR --out OUT` has the backend open and save each `.xlsx`/`.xlsm` file, and
    records `saved` or `refused` with Excel's error in `resave.json` (test with a fake backend).
- **Probe** (docs/research/xlsx-round-trip/excel-resave-probe-2026-11-07.md, reproducible with `resave_probe.py`).
  Excel for the web opened all 104 patched saves: the fixtures R0, R1/E6 and R1/all, and a 24-case corpus sample R0
  and R1/all. 51 of the 52 R0 saves match the source after Excel's save. All 9 refusals are regenerated saves:
  - 8 drop the external-link parts. The existing T-4143 now has this evidence.
  - 1 writes an empty `dcterms:created`. Filed as T-4909.
- **Semantics.** docs/excel-semantics/xlsx-storage.md:
  - "A patched package keeps the source's hints" is now VERIFIED for every hint that was exercised.
  - New: "An empty W3CDTF date makes Excel refuse the package".
  - New: "A [n] reference without its externalLink part makes Excel refuse the package".
- **Follow-ups.**
  - T-4909: core.xml omits an empty created date and escapes the creator.
  - T-4910: importer diagnostics go to stderr. Today they break `package-diff --json` on some files.
  - Still unexercised: a dropped calcChain, emptied rows, desktop Excel.
