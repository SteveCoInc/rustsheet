---
id: T-4961
title: "Round-trip ratchet: a case whose checked count changed must not hide a regression in its unchanged scenarios (54436 under #3892)"
milestone: M15
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
Adding a round-trip scenario, or anything else that changes a case's checked count, can no longer record an
existing scenario's regression as new coverage. A failing item in a scenario whose own checked count held is a
regression, as SPEC-0033's ratchet means it to be.

## Context
- `baseline::diff_items` (crates/rustsheet-eval/src/baseline.rs) treats every new failing item of a case whose `C`
  count changed as new coverage, not as a regression. The round-trip suites keep one `C` count per case, summed
  over all scenarios.
- #3892 (T-4514, a2bae4dec) added scenario R1 and re-recorded every round-trip baseline. That doubled
  `C roundtrip-poi/54436.xlsx` from 995 to 1990, so `update-baseline` recorded
  `F roundtrip-poi/54436.xlsx * 296 2cd19df138a87ec5` without complaint. The digest is the one R0 alone reported as
  `REGRESSION` before #3892 (T-4867, T-4825): the line is T-4176's (#3800) R0 regression, not an R1 finding.
- After that re-record, `roundtrip --check` passes on main with the regression in the baseline. The nightly gate
  (decision 0323) and a gating PR CI job (T-4726) both stay green over it. T-4825 owns 54436 itself and checks
  the other 331 `F` lines #3892 added. This task closes the mechanism.
- Cells mode has the same rule: a changed case's new failures are new coverage. That is intended where the checked
  cells really changed. The round-trip scenarios are separable, so the count can be kept per scenario.

## Acceptance criteria
- [x] A spec change (SPEC-0033 "Ratchet", by decision record if it changes the baseline format) says how a case's
      checked count is kept per scenario. One way: `C` lines carry the scenario, or a `cases`-mode `F` line keeps a
      digest per scenario. A failing item in a scenario whose own count held is then a regression.
- [x] `rustsheet-eval roundtrip --check` and `--update-baseline` follow it. A unit test adds a scenario to a case
      whose R0 already fails a new item, and the R0 item is refused as a regression.
- [x] The existing baselines are re-recorded in the new form with no regression accepted. The PR names any `F`
      line that cannot be carried over unchanged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing 54436 (T-4825) and gating the corpus round-trip suites in PR CI (T-4725, T-4726).
- The cells-mode ratchet.

## Outcome
- **Decision 0345, and SPEC-0033's "Baseline" bullet:** round-trip baselines are keyed `<case>#<scenario>`, in
  `cells` and `cases` mode alike. Each key has that scenario's own `C` count and its own `F` lines.
- **Code:** `crates/rustsheet-eval/src/roundtrip/by_scenario.rs` builds them. `roundtrip.rs` writes and judges
  that form, and judges a committed file still in the per-case form with the old rule (`roundtrip::baseline`), so
  the migration re-record refuses anything the old rule refused.
- **Tests (`by_scenario::tests`):**
  - #3892's case, both modes: R1 is added while R0 newly loses an item. The old rule passes it, as recorded; the
    new rule reports one regression at `poi/a.xlsx#R0` and the new key `poi/a.xlsx#R1`.
  - A new scenario's own failures are new coverage.
  - Keys carry each scenario's own count.
  - A case that did not run fails on `#R0`.
- **Migration:** all five round-trip baselines were re-recorded with `rustsheet-eval roundtrip --update-baseline
  --timeout 300`.
  - Before re-recording, I refetched calipers: my cache predated its suite's include list, so the first attempt
    refused (`REMOVED`) exactly as it should.
  - A check script compared old and new files. Each case's counts, summed over its scenarios, equal the old count.
  - Cells mode (fixtures, excel-saved): the failing items are identical.
  - Cases mode (poi, corpus-bench, calipers): the failing cases are identical, and every case failing in one
    scenario keeps its old line byte for byte (140/140, 243/243 and 164/164).
- **54436:** the line is now `F roundtrip-poi/54436.xlsx#R0 * 296 2cd19df138a87ec5`. It is still T-4825's to fix.
- **Check:** `roundtrip --check` passes on the migrated files.

