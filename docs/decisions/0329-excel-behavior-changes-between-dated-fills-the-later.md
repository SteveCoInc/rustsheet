# 0329 — Excel behavior changes between dated fills: the later reproduced fill is the target

- **Status:** accepted (2026-11-07, T-4661)
- **Context:**
  - CLAUDE.md directive 1 sets the bar at *current* Microsoft 365 Excel. The oracle (excel-graph, Excel for the web) is
    a live service, so the same input bytes can give a different value on a later date because Excel changed.
  - T-2609 found such a change. Byte-identical copies of the October nested-cell probes were refilled on 2026-11-06
    (`<case>-refill-1106`). They agree with their October fills except in the reads of a nested GROUPBY or PIVOTBY
    cell (docs/excel-semantics/arrays.md "Operators over nested cells, and Excel's change of 2026-11"):
    - SUM over a result holding nested cells is `#N/A`; it was a flattened sum. COUNT skips the nested cells, and
      COUNTA counts each as one value. Evidence: t2592-nested-groupby-cells-refill-1106 A6:A8,
      t2594-nested-aggregates-refill-1106 A26:A29, and t2625-nested-consumers-refill-1106 A13, A17, A20, A25, A26.
    - An element-wise lift reads a nested cell as an error, not as its top-left. Evidence:
      t2360-array-lambda-holidays-refill-1106 P!C1, C9, C17, C25, C33 and
      t2311-pivotby-value-columns-error-refill-1106 P!C12.
    - `&""` over an INDEXed row makes each nested cell `#VALUE!`. Evidence: t2625-refill-1106 A32 (the October fill was
      one `#N/A`).
  - The t2592 and t2594 copies were refilled twice on 2026-11-06, and the two fills are identical. One rule explains
    every changed cell across five probes. The new T-2609 probes, filled the same day, agree with it
    (t2609-nested-operators-and-lifts A44:A47, -one-cell A27). So this is a behavior change, not variation between
    fills.
  - The opposite kind of case is a single cell whose value flips between fills of the same bytes with no rule behind
    it: gen-m9 d912671baa1364aa Calc!A76 and probes/amordegrc-fraction-state-ai Sheet1!A1. Those are T-3610's question,
    and decision 0284 kept their old values committed as known failures. T-3610's draft rule rules out re-filling
    until a run matches us.
  - Every fill on both dates is excel-graph. No desktop fill exists (decision 0039).
- **Options considered:**
  - *(a) Keep the October fill as the target until a desktop or second-backend fill confirms the change.* Decision
    0039 makes excel-graph the oracle of record for Phases 1–3, and it lets no task wait on a desktop confirmation.
    With this option the engine would knowingly follow an Excel that no longer exists.
  - *(b) Replace the October sidecars with the refills.* This would overwrite committed evidence with a later run and
    hide the regression. Decision 0284 rejects it for the same reason.
  - *(c) The later fill is the target once it is shown to be a change; the earlier fill stays committed and is scored
    as a named known failure* (chosen).
  - *(d) Exclude the superseded cells from scoring.* This needs a new harness mechanism, and it would make the cells
    invisible to the gap list. T-3610 may still choose it for variance cells; this record does not need it.
- **Decision:**
  1. **When a fill is a change.** A later fill's value for a cell is a behavior change when all of these hold:
     - a byte-identical copy of the input (same sha256, or a case that builds the same bytes) gives the new value;
     - either two fills of that copy agree, or one rule, stated in docs/excel-semantics/, explains the changed cells
       of at least two probes;
     - the copy's other cells agree with the earlier fill.
     Any other difference between fills is variation, and T-3610 decides how it is scored.
  2. **Target.** For a behavior change, the later fill is the target (directive 1, "current"). The engine moves to it
     in the task that implements the change. No desktop confirmation is needed (decision 0039 item 2). If a later
     desktop fill disagrees, that is a web-vs-desktop conflict and goes to M14 (decision 0039 item 4).
  3. **Superseded cells.** The earlier sidecar stays committed and unchanged. Its changed cells are *superseded*:
     - When the engine moves to the later value, the implementing task records each superseded cell as a known
       failure. It runs `rustsheet-eval update-baseline --allow-regressions` once, for exactly the cells this record
       or its semantics entry lists, as decision 0284 did. Reviewers confirm that the baseline diff adds no other F
       line.
     - The semantics entry lists the superseded cells next to the refill cells that replace them.
     - No one re-fills or edits the earlier sidecar to clear them.
  4. **This change.** The 2026-11-06 nested-cell reads are a behavior change under item 1, and they are the target.
     SPEC-0018 rules 4 and 7 and its "One reader" are revised to them (T-4661). The superseded cells, all in
     probes, are:
     - t2592-nested-groupby-cells A6:A8;
     - t2594-nested-aggregates A26:A29;
     - t2625-nested-consumers A13, A17, A20, A25, A26 and A32;
     - t2360-array-lambda-holidays P!C1, C9, C17, C25 and C33;
     - t2311-pivotby-value-columns-error P!C12.
     The October cells of probes/t2475-stat-subexpressions P!A62, A63 and gen-m9/17c110a53924beb8 Calc!A300 are
     superseded too. Ours already gives their November values, so they are F lines against the October values today.
     They stay F lines, and they need no regression run.
- **Consequences:**
  - T-4736 owns the aggregate reads: SUM, COUNT and COUNTA, the t2592, t2594 and t2625 cells except A32, and SUM over a
    reference-body result (t2609 A50).
    T-4737 owns the element-wise lift and operator reads: the t2360, t2311 and t2625 A32 cells, t2609 A44:A46,
    -one-cell A27 and the t2475 refill's P!A64. T-4738 owns the lifted INDEX top-left (t2609 A56:A59). Each task's PR
    makes the one allowed regression run for the superseded cells it lists.
  - Known failures fall by the refill cells each task fixes and rise by the October cells it supersedes; the gap
    list shows both. No gate or script changes.
  - A later change of the same kind follows items 1–3 with no new decision record. It needs a semantics entry that
    lists the superseded cells, and the implementing task cites this record for its allowed regressions.
  - T-3610 keeps the variance case (a cell whose value differs with no rule behind it). If it chooses to exclude
    cells from scoring, it may extend that choice to superseded cells; until then they stay F lines.
