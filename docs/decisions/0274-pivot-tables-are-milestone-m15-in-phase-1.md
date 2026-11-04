# 0274 — Pivot tables are milestone M15 in Phase 1

- **Status:** accepted (2026-11-04, owner request, T-4120). It amends 0027 (phase order): M15
  joins Phase 1's milestone list, after M11.
- **Context:**
  - Today the engine imports pivot caches and definitions read-only and answers GETPIVOTDATA (SPEC-0015).
    It never renders, refreshes or exports a pivot.
  - On 2026-11-04 the owner asked for a spike into pivot tables and PivotCharts. The research and the
    plan are in docs/research/pivot-tables-2026-11-04/.
  - The spike showed that the excel-graph oracle renders pivot definitions we author in the file:
    layouts, subtotals, show-as, calculated fields, Top-N and grouping (oracle-spike.md). Every pivot
    feature therefore gets oracle evidence the way functions do.
  - After reviewing the plan, the owner chose a new milestone inside Phase 1 and asked that every task be
    filed.
- **Decision:**
  1. **M15 "Pivot tables" is a Phase 1 milestone**, listed after M11 in docs/milestones.md. Phase 1 does
     not exit until M15's exit criteria hold.
  2. **No new lane.** M15 is staged PV0–PV5 as the plan's §8 says, and its tasks use the existing lanes:
     - `engine` for the pivot model, refresh, result, layout, export and PivotChart data;
     - `harness` for the builder, sidecars, scoring, suites and the catalog;
     - `research` for probes;
     - `functions-lookup` for GETPIVOTDATA;
     - `api`;
     - `quality`.

     This avoids changing the routine allowlists.
  3. **The pivot feature catalog (eval/catalog/pivot-features.tsv) is M15's scope list,** as the
     function catalog is for the other Phase 1 milestones. A row is in scope unless a decision rules it
     out.
  4. **Ruled out now:**
     - OLAP, the Data Model and Power Pivot (DAX, Distinct Count);
     - external and consolidation sources;
     - styles and conditional formats beyond round-trip preservation;
     - UI-only commands (Show Report Filter Pages, field-drop auto grouping).

     **PivotCharts are in scope** as data and file round trip only: RustSheet draws no charts.
  5. **The oracle of record is current Excel for the web** (0039). Behaviour still rolling out in 2025–26
     (PivotTable Auto Refresh, pivot `#SPILL!`) is matched as the web shows it, with the date it was
     observed. Rows the web cannot render go to M14 and never block M15.
  6. **Exit:** for every stage, each in-scope catalog row has at least 30 oracle cells; the `pivots`,
     `gen-pivot` and fresh `pivot-corpus` cells are not visibly off (0083), apart from listed deviations;
     every remaining row has a task or an out-of-scope ruling. The report's M15 row measures this. Until
     the row exists the milestone counts as not met.
- **Consequences:**
  - M15 tasks are filed under docs/tasks/M15 from the plan's task list.
  - Implementation follows SPEC-0030, written as its own task.
  - scripts/phase1_exit.py gains M15 with the report row (its own task); until then the script still
    lists M1–M11.
  - Because M15 comes last in Phase 1's order, workers reach it after the earlier milestones unless the
    owner or the steward puts its lanes on the critical-lane line. The owner works it directly from the
    start.
