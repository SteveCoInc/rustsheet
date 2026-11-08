# 0352 — R2 sheet-edit scenarios enter the round-trip baselines as new coverage

- **Status:** accepted (2026-11-08, T-5005)
- **Context:**
  - T-5005 adds SPEC-0033's sheet-level R2 scenarios: `R2/rename`, `R2/add`, `R2/delete` and `R2/move`. The
    fixtures suite runs all four, and the corpus suites (calipers, corpus-bench, poi, excel-saved) run `R2/rename` and
    `R2/delete`. Each is scored by D5's expected-delta rule against R0, with its own `Δ(E)`.
  - A sheet edit leaves the verbatim and patch tiers (SPEC-0034 D1), so every R2 save is regenerated. R0 of almost
    every case saves verbatim and keeps everything. R2's violations are therefore mostly what the regenerated export
    loses: the theme, `fileVersion`, `calcPr@calcId`, `pageMargins`, `sheetFormatPr`, docProps fields, and the
    unmodelled parts stage 5 does not keep yet. SPEC-0034 D5.7 names these scenarios as the measure of stage 5.
  - A deleted sheet's own parts are lost until the save report names them `OwnerDeleted` (T-5004). Only then do they
    pass as `dropped` (SPEC-0033 D7.2).
  - Per decision 0345 a new scenario is a new baseline key, so `roundtrip --update-baseline` records these failures
    as new coverage without `--allow-regressions`. T-5005's acceptance asks for a record whenever the scenarios add
    failing items, and they do: RATES.
- **Decision:** the R2 sheet-edit scenarios' failing items enter the round-trip baselines as known failures,
  recorded by a plain `roundtrip --update-baseline` (no `--allow-regressions`). No R0 or R1 key changes.
- **Consequences:**
  - The baselines gain `#R2/...` keys only. Reviewers confirm the diff touches no other scenario's line.
  - Stage 5 work (T-5000 to T-5004, T-5008 to T-5010) burns these down by re-recording: each kept part or reported
    `OwnerDeleted` drop removes lines.
  - A new R2 violation in a scenario whose count held is a regression, as for every other scenario.
