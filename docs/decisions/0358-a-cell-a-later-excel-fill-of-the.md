# 0358 — A cell a later Excel fill of the same inputs superseded is scored against that fill

- **Status:** accepted (2026-11-08, owner ruling; T-3610). Amends decision 0329 item 3 and decision 0284; resolves
  T-3610 for the cells it lists.
- **Context:**
  - Directive 1 sets the bar at *current* Excel, and the oracle (excel-graph) is a live service: the same input bytes
    can give a different value on a later date. Two kinds are known:
    - a behavior change (decision 0329): the October nested-cell reads of GROUPBY and PIVOTBY, refilled on 2026-11-06
      and 2026-11-07, where one rule explains every changed cell and fresh probes confirm it;
    - an earlier fill that later fills of the same bytes do not reproduce (T-3610): probes/t4618-order-after-active-
      backward Q!A1 (`#NAME?` on 2026-11-06; 1 in one 2026-11-07 fill, committed as two copies of one upload, and in five
      one-variable variants filled the same day),
      gen-m9 d912671baa1364aa Calc!A76 (1 on 2026-10-28, 0 on 2026-11-01) and probes/amordegrc-fraction-state-ai
      Sheet1!A1 (0 on 2026-10-30, 485 in both 2026-11-05 fills).
  - Decision 0329 made the later fill the target but kept the earlier sidecar committed *and scored*, so each
    superseded cell stayed a known failure by design. Decision 0284 did the same for the AMORDEGRC cells. The engine
    gives the later value in every one of them, yet they count as visibly off: 19 of M7's 43 visibly-off probe cells and
    M8's t4618 cell, all of which no engine change can pass, so the M7 and M8 exit rows could never be met.
  - The owner asked (2026-11-08) for these cells to be scored against the Excel answer that reproduces.
  - Decision 0329 weighed excluding such cells from scoring (its option (d)) and rejected it, because they would
    vanish from the gap list. This record does not exclude them: each is still scored, against a golden fill of the
    same inputs, so a cell the engine gets wrong against the later fill still shows. That keeps 0329's reason and
    drops only its cost, a cell no engine change can pass.
- **Decision:**
  1. A spec suite may name a `superseded` file in its manifest (`suite.toml`). Each line names a scored cell and the
     same cell in a later golden sidecar (`case<TAB>Sheet!A1<TAB>sidecar<TAB>Sheet!A1`, the `cached_overrides` format,
     decision 0032). The eval scores the cell against that later value (`runner/prepare.rs` `apply_superseded`).
  2. A line is valid only when the later sidecar is golden (excel-graph or excel-desktop), cites the same cell, and its
     case is a copy of the cell's case: every key but description, tags, check, expected and expected_source equal
     (`overrides::same_inputs`, decision 0329 item 1's copy rule). check, expected and expected_source only choose
     which cells a case scores and where a hand value comes from; the superseded value always comes from the golden
     sidecar, so a copy may differ there (the AMORDEGRC bisect copy checks fewer cells). The eval refuses any other
     line, and a line for a cell that is not scored, so a stale or mistaken line fails the run. A `superseded.tsv` is a
     protected path (`scripts/check_governance.py`, as a `deviations` file is): a PR that changes it needs a decision
     record, so no line lands without one. A line whose case path names no case of the suite is not detected (the
     same gap as `cached_overrides` and `deviations`); a line for an unscored cell of a named case is.
  3a. **When a cell is superseded.** A *copy* is a case whose JSON equals the cell's case once the keys of item 2 are
     removed, whatever its bytes hash to (the workbook built from it may differ in metadata). A cell qualifies only by
     one of these, stated in the record that adds its line:
     - a behavior change under decision 0329 item 1 (two agreeing later fills of a copy, or one rule in
       docs/excel-semantics/ confirmed by a newly written probe cell filled on the later date); or
     - an earlier fill that does not reproduce: at least two later fills of a copy agree with each other and differ
       from it, or one later fill of a copy agrees with fresh one-variable variant probes filled the same day.
     Every fill of a copy that is made is committed, whatever it shows: a fill may not be dropped, and if any committed
     later fill disagrees with the target the cell is variance under item 5, not superseded. So re-filling until a run
     matches the engine is ruled out: the run that does not match is committed too and blocks the line. A fill's target
     is never the engine's value: the later fills decide it whatever the engine gives. Fills made before this record count as the
     record that reports them states them (decision 0284 reports the uncommitted 2026-11-05 refill of -ai beside its
     committed copy -av); from this record on, the commit rule above applies.
  3. A line is added only for a cell a decision record names as superseded or as not reproduced: decision 0329's
     cells (the t2592, t2594, t2625, t2360, t2311 and t2475 probes, gen-m9 17c110a53924beb8 Calc!A300,
     gen-m9 e85daedce5c2675e Calc!A35, probes/t2436/e85daedce5c2675e Calc!A31 and A46, gen-m7 21b4eef18424ce94
     Calc!B1505:E1514), and this record's T-3610 cells (t4618 Q!A1, gen-m9 d912671baa1364aa Calc!A76,
     amordegrc-fraction-state-ai Sheet1!A1). 55 lines in all: eval/suites/probes, gen-m7 and gen-m9 `superseded.tsv`.
  4. The earlier sidecar stays committed and unchanged (decision 0329 item 3). Nobody re-fills or edits it. A later
     change of the same kind adds lines through a decision record that names the cells, as decision 0329's
     consequences already require.
  5. For a cell that varies between fills with no later fill reproducing either value, T-3610's question stays open:
     it is not superseded, and it stays a named known failure. Decision 0112's LN FYL2X near-ties
     (probes-numeric/t2143-ln-near-ties-fill1..3, three fills that disagree with each other) are such cells and stay
     named known failures.
- **Consequences:**
  - The 55 cells score against current Excel. The engine gives each later value, so they leave the baseline's F lines
    (a fix, recorded by `make eval-baseline`), and the M7 and M8 probe rows drop by 19 and 1.
  - Decision 0329 item 3's "recorded as a known failure" and its `--allow-regressions` run no longer apply to cells
    listed here; a future superseding task adds its lines instead.
  - T-4913's owner-tool gap closes for these cells: a passing cell needs no owner.
  - Scoring is not weakened: each listed cell is still scored against a golden Excel value of the same inputs, and the
    harness rejects any line that is not.
