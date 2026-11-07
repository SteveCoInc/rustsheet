---
id: T-4835
title: "IRR solver lineage table: VB runtime, RATE, LibreOffice, Gnumeric, POI, historical docs"
milestone: M9
lane: research
status: in-progress
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
- [ ] docs/research/irr-2026-11-07/lineage.md: rows = mechanisms (variable, start, offset, anchor, stop tests and
      their scaling, flat handling, pole handling, cap and cap return, error codes, leading zeros), columns = sources,
      each cell marked Excel-evidenced, contradicted or unknown, with links.
- [ ] A ranked list of mechanisms to test at each event, fed to T-4836.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Scoring (T-4836).
