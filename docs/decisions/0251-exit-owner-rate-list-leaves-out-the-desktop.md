# 0251 — Exit-owner rate list leaves out the desktop-owned cells the Phase 1 rate leaves out

- **Status:** accepted (2026-11-02, T-3805). Applies decision 0143 to `make exit-owners`.
- **Context:**
  - Decision 0143 leaves a visibly-off cell out of the Phase 1 rate when only open M14 tasks own it. `make eval`
    writes those cells to `eval/runs/desktop-owned.tsv` (`scripts/desktop_owned.py`) and the Rust rate reads that
    file (`rustsheet-eval run --desktop-owned`).
  - `scripts/exit_owners.py` listed M9's `rate:<FUNCTION>` cells after dropping only the `r3_pass` and `phase1b`
    classes of the report's `r1c_left_out`, which marks `m14_owned` only on probe-row cells (decision 0188 R7).
    So gen-m9 cells the rate already leaves out were listed as unowned exit cells: on origin/main 56917e82c,
    272 `rate:FORECAST.ETS*` cells, exactly the four functions' `desktop_owned` counts in
    `v15_checkpoint.functions`. That invites triagers to file duplicate tasks for cells that block no exit.
- **Decision:**
  - A `rate:<FUNCTION>` exit cell is listed only when the Phase 1 rate counts it: besides the `r3_pass` and
    `phase1b` classes, a cell on `eval/runs/desktop-owned.tsv` is left out. The list is read from that file, the
    same one the rate reads; ownership is never recomputed in `exit_owners.py`. A missing file (no `make eval` yet)
    leaves nothing out.
  - The other criteria (gen suite, ironcalc-calc rows, probe rows) do not change.
- **Consequences:**
  - `make exit-owners MILESTONE=M9` no longer reports desktop-owned FORECAST.ETS-family cells as unowned; outputs
    for the other milestones are unchanged (they have no `rate:` criterion).
  - Which cells are desktop-owned stays `desktop_owned.py`'s rule; a change there flows to both the rate and this
    list.
  - Tests: `scripts/test_exit_owners.py` (a below-rate function's cell on the list is not listed; one off it is).
