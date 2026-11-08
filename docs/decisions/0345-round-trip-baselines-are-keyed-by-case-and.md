# 0345 — Round-trip baselines are keyed by case and scenario, so a new scenario cannot hide an existing scenario's regression

- **Status:** accepted (2026-11-07, T-4961)
- **Context:**
  - The ratchet (`baseline::diff_items`) counts a failing item as a regression only when its case's checked count
    is unchanged. A failing item of a case whose count changed is new coverage, so it is allowed.
  - A round-trip case had one `C` count, summed over the scenarios it ran (SPEC-0033 D8).
  - #3892 (T-4514) added scenario R1, so every case's count changed. Its re-record then took
    `roundtrip-poi/54436.xlsx`'s R0 loss, which #3800 (T-4176) had caused, for new coverage:
    `F roundtrip-poi/54436.xlsx * 296 2cd19df138a87ec5`. R0 alone had reported exactly that digest as
    `REGRESSION` before. Every gate then passed with the loss recorded (T-4825, T-4867).
  - The scenarios are separate runs, each with its own count (`CaseResult::scenarios`), so their counts can be
    kept apart.
- **Decision:**
  - **Keys:** every line of a round-trip baseline (`eval/baseline/roundtrip-*.txt`), in `cells` and `cases` mode,
    is keyed `<case>#<scenario>`. Each scenario's `C` line gives that scenario's own count, and its `F` lines
    give its own failing items (`cells`) or its own `* <n> <digest>` (`cases`). A case that did not run is one
    `!Status` failure on its first scenario, `R0`.
  - **Ratchet:** the ratchet and SPEC-0033's round-trip rules (a count that falls, a case that newly does not run,
    a `cases` line whose n held or rose) apply per key. A new scenario is a new key, so its failures are new
    coverage. A failing item in a scenario whose own count held is a regression.
  - **Migration:**
    - A committed baseline without `#` in its keys is judged with the old per-case rule (`roundtrip::baseline`).
      The re-record that writes the new form therefore refuses anything the old rule refused.
    - T-4961 migrated all five round-trip baselines that way. Summed over its scenarios, each case's count equals
      its old count, and its failing items or failing cases are unchanged.
- **Consequences:**
  - The hole is closed for round trips: adding or changing one scenario no longer re-records another scenario's
    loss.
  - The migration accepted no regression, so `F roundtrip-poi/54436.xlsx#R0 * 296 2cd19df138a87ec5` stays a known
    failure. T-4825 owns it, and its fix removes the line by a re-record.
  - Cells-mode suites (`eval/baseline/<suite>.txt` for the cell suites) keep their per-case rule: there a changed
    checked-cell count really is changed coverage.
  - Baseline files are a little longer: one `C` line per scenario of each case.
