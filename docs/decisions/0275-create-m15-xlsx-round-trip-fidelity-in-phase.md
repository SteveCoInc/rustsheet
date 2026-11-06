# 0275 — Create M15 xlsx round-trip fidelity in Phase 2

- **Status:** accepted (2026-11-04, owner review of docs/research/xlsx-round-trip-2026-11-04.md)
- **Context:**
  - Two scope lines have no exit criterion, spec, task or suite behind them: M11's "xlsx
    round-trip preservation of unknown parts" (Phase 1) and M10's "Style storage and round-trip".
  - `docs/architecture/overview.md` claims an "unknown-part passthrough" that does not exist.
  - The 2026-11-04 census loaded and re-saved 2,028 corpus workbooks. Every part, element and
    attribute the typed model lacks was lost in 100% of the files that had it: data validation,
    page setup, comments, charts, drawings, VBA, pivots, external links, custom properties, every
    unknown `extLst`. Chartsheets lose the whole sheet.
  - The owner ruled that this work is the most important after Phase 1, must not take priority
    over exiting it, and accepted recommendations D1–D6 of the research document.
- **Decision:**
  - Create **M15 "xlsx round-trip fidelity"** in Phase 2, worked alongside M3, M4 and M13. Its
    tasks stay `blocked` until Phase 1 exits. Measurement tasks get no Phase 1 carve-out (D2).
  - Move the two orphan scope lines into M15.
  - Cell-content import and export bugs the census found stay ordinary M11 tasks, because the
    cells they get wrong are Phase 1 parity.
  - The design direction is the research document's §5, adopted by D3–D6:
    - typed where modelled and verbatim where not, via an ownership matrix, a closure rule and
      bounded retention;
    - index-stable output, verbatim untouched parts and a cell patcher for content-only edits
      (D3);
    - range-bound kept content dropped and reported on a structural edit, never kept stale (D4);
    - VBA carried byte-for-byte into macro-enabled files (D5);
    - `x15ac:absPath` kept as Excel keeps it, with an option to strip it (D6).
  - D7 (whether Common Crawl workbooks may go to the oracle) stays open.
- **Consequences:**
  - `docs/milestones.md` gains M15, with the exit criteria from research §7, and loses the
    orphan lines in M10 and M11.
  - `docs/architecture/overview.md` no longer claims passthrough.
  - M15's two specs (the round-trip contract and the ownership matrix) and its stage 0
    measurement tasks are filed `blocked`.
  - Stages 1–5 are decomposed from the research document by the planner once those specs land.
