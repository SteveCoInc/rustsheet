# 0308 — Phase 1 rate leaves out FORECAST.ETS family and desktop-only CELL info types (web cannot evidence them)

- **Status:** accepted (2026-11-06, owner request, T-4207, T-3011). The owner, in an interactive Claude Code session:
  "put the desktop only functions on the exception list".
- **Context:**
  - Decision 0266 made desktop-Excel work out of scope and dropped the M14 tasks that owned these cells, so
    `scripts/desktop_owned.py` (decision 0143) no longer leaves them out and they count against the Phase 1 rate
    again (T-4207).
  - FORECAST.ETS, FORECAST.ETS.STAT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.CONFINT: Excel for the web returns
    `#N/A` for every call that passes the argument step (statistics.md "FORECAST.ETS family: excel-graph never
    computes it"; re-probed by T-3641 on 2026-11-05). The engine's model (decision 0221) stays as implemented
    (decision 0266 rule 2). `make eval` on c97e0b9c: 402 visibly-off cells call the family and expect the web's
    `#N/A` (gen-m9 326, probes 76). All four functions are below Gate B's rate only because of these cells.
  - CELL with info_type "color", "filename", "format", "parentheses", "prefix", "protect" or "width": Excel for the
    web returns `#VALUE!` for each (probes/t3011-cell-format-info-types, excel-graph 2026-11-06, 38 of 38 cells
    `#VALUE!`, every info type except "width", which the oracle's canary refuses because the web does not implement it;
    `tools/src/rustsheet_tools/oracle/canary.py`). The engine returns `#VALUE!` too, so it matches the oracle of
    record. The 153 visibly-off CELL cells (calipers goldens formulas_information and pv_goldens) carry desktop
    Excel's cached values (a width, a format code, a file path), which no web evidence can reach. CELL is below
    Gate B's rate (149 of 489 cells) only because of them.
- **Options considered:**
  - *Count them like any other visibly-off cell.* The four FORECAST.ETS functions and CELL stay below the Phase 1 rate
    for good: no fix can move them, since the engine already matches the web where the web answers.
  - *List them as Phase 1 rate exceptions (chosen),* the mechanism decision 0134 built for desktop-only evidence.
- **Decision:**
  1. The 555 cells (402 FORECAST.ETS-family, 153 desktop-only CELL) are lines of `eval/milestones/phase1-exceptions.tsv`
     under this decision, owned by T-4373 (the open placeholder that refills them if a desktop or web oracle ever
     computes them). They leave the Phase 1 rate per decision 0134 and stay known failures; nothing else is hidden.
  2. gen-m9/fa25422d6f18c157 Calc!A75 (Excel `#REF!`, web-observable) is not listed (T-4208).
  3. If Excel for the web ever computes the FORECAST.ETS family or these CELL info types (T-3641's re-probe found it
     does not, 2026-11-05), the cells become fixable: their lines go and the engine follows the web.
  4. New cells of the same kind (a FORECAST.ETS-family call where the web gives `#N/A`, or a CELL info type above
     where the web gives `#VALUE!` and a desktop-authored workbook caches a value) may be added by a later PR citing
     this decision, each with its own oracle evidence (the web's `#N/A` or `#VALUE!` sidecar value) in the PR.
- **Consequences:**
  - FORECAST.ETS, FORECAST.ETS.STAT, FORECAST.ETS.SEASONALITY, FORECAST.ETS.CONFINT and CELL should meet the Phase 1
    rate. `make eval` on bfc30b2a leaves out 564 cells and puts Gate B at 469 of 506 (456 on c97e0b9c, with other merged
    work in between); `make phase1-exit` owners met.
  - T-3011 and T-4207 are done (T-4207's code criterion is T-4527). T-4373 owns the listed cells, and the rows that
    named the dropped T-1075 now name T-4373. T-3004 is dropped.
