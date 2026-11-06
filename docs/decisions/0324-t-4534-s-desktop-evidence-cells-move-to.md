# 0324 — T-4534's desktop-evidence cells move to T-4373; SUMPRODUCT J1328 is a Phase 1 rate exception

- **Status:** accepted (2026-11-06, T-4534).
- **Amends:** decision 0308's consequence "the older rows that named the dropped T-1075 now name T-4534": they name
  T-4373 instead. No cell is added to or removed from either list's measure except J1328 (item 2).
- **Context:**
  - Decision 0308 moved the owner column of the lines that named the dropped T-1075 (decision 0266) to T-4534, an open
    M9 task, so that they kept an open owner. T-4534's job was to triage the cells T-4207 listed. Its two partial PRs
    (decisions 0315, T-4630) settled every cell but these, which need desktop Excel evidence and nothing else:
    - eval/milestones/exit-exceptions.tsv: IF_ARRAY C4, C11, E4 and ATAN2_POWER D14, D26 (decisions 0074, 0085, 0086:
      Mac desktop SIN(4) one ulp from the web, not visibly off); COMPLEXs N6, AL12, AL26 (decision 0188 R6) and K25,
      IRR Two Values!C4 (decision 0196): the engine follows Excel for the web, the workbook caches Mac 16.0300 digits.
    - eval/milestones/phase1-exceptions.tsv: the poi EverythingTests J136, G936, H936, K936 (decision 0134) and the
      COMPLEXs and IRR cells above.
    - probes/forecast-ets-longer-series Sheet1!K1:K36, K39:K42: already decision 0308's lines, owned by T-4373.
  - Each list's header says its owner column names the task that owns the cell; phase1-exceptions.tsv says "the open
    M14 task". T-4373 is the one open M14 task and already owns decision 0308's and 0315's desktop-only cells. Keeping
    T-4534 open only to hold an owner column would leave an M9 task that no work can close.
  - poi/FormulaEvalTestData_Copy.xlsx EverythingTests!J1328 `SUMPRODUCT(G1330:G1330,I1330:I1330)` (G1330 the text "4",
    I1330 the number 2): Excel 15 (AppVersion 15.0300) caches `#VALUE!`; Microsoft 365 (excel-graph,
    probes/t2921-sumproduct-one-cell-numeric-text A45, `SUMPRODUCT` of the same shape) gives 0, as the engine does
    (coercion.md "Numeric text in a one-cell argument", CONFLICTING). It is the shape of J136 under decision 0134: an
    Excel build below 16 does not outrank the web (decision 0030 rule 2), so in Phases 1–3 no fix exists and only a
    16.x desktop value can settle it (decision 0039).
- **Options considered:**
  - *Keep T-4534 open as the owner.* An M9 task whose only content is cells no M9 work can move; it never closes.
  - *A new M9 placeholder task.* The same problem under another id.
  - *Move the owner column to T-4373 (chosen),* the open M14 placeholder for desktop-only evidence, and list J1328 under
    decision 0134's mechanism like J136.
- **Decision:**
  1. Every line of eval/milestones/exit-exceptions.tsv and eval/milestones/phase1-exceptions.tsv whose owner column is
     T-4534 names T-4373 instead. T-4373's task file lists those cells. Their decisions, milestones and measures are
     unchanged.
  2. poi/FormulaEvalTestData_Copy.xlsx!EverythingTests!J1328 is a line of eval/milestones/phase1-exceptions.tsv under
     this decision, owned by T-4373. It leaves SUMPRODUCT's Phase 1 rate per decision 0134 and stays a known failure.
  3. T-4534 is done.
- **Consequences:**
  - With T-4534 done, T-4373 is the only open task naming these cells, so decision 0143's desktop-owned list (the
    Phase 1 rate) and decision 0188 R1(c)'s `m14_owned` class read them as desktop-owned, which is what their
    decisions already say they are.
  - If desktop or web evidence ever settles a cell, its line goes (decisions 0087, 0134) and its fix gets a task.
