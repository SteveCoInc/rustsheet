---
id: T-4835
title: "IRR solver lineage table: VB runtime, RATE, LibreOffice, Gnumeric, POI, historical docs"
milestone: M9
lane: research
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A mechanism-by-source table of IRR-family solvers that ranks the candidate behaviours at the flat secant, the pole and the cap.

## Context
Plan: docs/research/irr-2026-11-07/plan.md (T-4833). Phase 1.
- dotnet/runtime Financial.vb (IRR, Rate, OptPV2) is read (plan Appendix A).
- To read: LibreOffice `ScInterpreter::ScIRR`, Gnumeric's irr and goal_seek, Apache POI `Irr`, formulajs, Lotus
  1-2-3/Multiplan @IRR documentation, Microsoft KB articles on IRR #NUM!, "Function Improvements in Excel 2010".

## Acceptance criteria
- [x] docs/research/irr-2026-11-07/lineage.md: rows = mechanisms (variable, start, offset, anchor, stop tests and
      their scaling, flat handling, pole handling, cap and cap return, error codes, leading zeros), columns = sources,
      each cell marked Excel-evidenced, contradicted or unknown, with links.
- [x] A ranked list of mechanisms to test at each event, fed to T-4836.
- [x] `make check` green; reviewers approve.

## Out of scope
- Scoring (T-4836).

## Outcome
docs/research/irr-2026-11-07/lineage.md: the mechanism-by-source table (17 mechanisms against Excel's IRR, Excel's
RATE, the VB runtime's IRR and Rate, LibreOffice, Gnumeric, Apache POI, formulajs and IronCalc upstream), every cell
marked E, C or ?, with source paths and line numbers, and the ranked mechanisms per event for T-4836 (flat: H1, H3,
H2; pole: H4, H5, H6; cap: H7, then no fixed cap; across events H10, H8, H9). Findings: the five non-Microsoft engines
are all Newton (a different family); VB's Rate maps a twice-flat secant to divide-by-zero, a concrete form of H10;
Microsoft's "Changes made to Excel functions" page lists IRR and XIRR (not RATE) among the functions whose algorithm
changed in Excel 2010. plan.md's Phase 1 source table points at the new page. Not reached: the Lotus 1-2-3 and Multiplan
manuals (not found) and KB Q70976 (host not reachable from the session). Eval delta: none (docs only).
