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
docs/research/irr-2026-11-07/lineage.md: 22 mechanism rows (method, variable, start, second point, anchor, pair update,
NPV form, step, function and zero-step stops, flat, pole, NPV overflow, cap, cap return, retry, second stage,
far-root bound, leading zeros, sign pre-check, guess range, error codes, history) against nine columns: Excel's
IRR and RATE (from financial.md's evidence) and the VB runtime, LibreOffice, Gnumeric, Apache POI, formulajs,
IronCalc upstream and Microsoft's docs, each read at a pinned commit. Every cell is marked E, C or U.
Findings: no other engine is a secant except the VB runtime; LibreOffice is the documented contract (20 Newton
steps, 1e-7 step), which Excel's evidence contradicts; Gnumeric and formulajs share Excel's forward running-product
NPV; POI's fixed-grid second stage is the only one that could be guess-free (T-4844); Microsoft lists IRR (with
RATE) among the Excel 2010 algorithm changes. The ranked lists per event go to T-4836, with four new candidates
H13 (RATE's keep-older-iterate, T-3963), H14 (reuse the slope), H15 (clamp inside the pole), H16 (POI's grid
bracket without the guess). Not reached: the Excel 2010 blog post (dead), KB Q75504 and Lotus @IRR docs (their
mirrors are blocked by the network policy). No oracle use, no code change; eval delta none.
