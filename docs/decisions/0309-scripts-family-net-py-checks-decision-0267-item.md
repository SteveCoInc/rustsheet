# 0309 — scripts/family_net.py checks decision 0267 item 6 (a) to (c)

- **Status:** accepted (2026-11-06, task T-4159).
- **Amends:** decision 0267 item 6 (d), which now names the script. Its conditions do not change.
- **Context:**
  - Decision 0267 item 6 lets a PR record incomplete-beta family regressions with `update-baseline
    --allow-regressions` under conditions (a) to (d). Only the Excel-fidelity reviewer checked them, from the PR body's
    arithmetic. The #3137 review (round 3, MINOR) asked for a tool, so the check can be reproduced.
  - `scripts/` is a protected path, so a new script there needs a record. It belongs there with the other
    report.json readers the steward and reviewers run (`exit_owners.py`, `visible_lock.py`, `new_case_owners.py`):
    standard library only, tested by `unittest` in `make check`.
- **Decision:**
  - `scripts/family_net.py MAIN_REPORT BRANCH_REPORT` checks (a) to (c) and prints the counts (d) cites. It exits 0
    when they hold, 1 when one fails, and 2 when the reports cannot be compared.
  - A cell regresses when its closeness bucket gets worse. That covers the baseline's newly failing cells and a
    family cell moving from `digits15` to `residue13` or past it. Text checks that start failing, and cases that stop
    running, are regressions too. Cases whose checked-cell count changed, and cases on one side only, are new
    coverage and are not compared.
  - The script derives no bucket and no family membership itself, so `closeness.rs` stays the one definition
    (CLAUDE.md directive 3). report.json now writes these on each listed cell (`closeness::serialize_cells`):
    `closeness`, the bucket, and `thirteen_digit_family`, from `blamed` and the anchor rule. It also gives the family
    list at the top level (`thirteen_digit_family`). The script reads (c) from `functions[].closeness`. An unlisted
    cell counts as bit-exact. Each case's `checked - passed - deviations` must equal its listed scored cells, or the
    script exits 2.
- **Consequences:**
  - A PR using item 6 runs `make eval` (or the same `rustsheet-eval run` suites) on main and on the branch, then
    pastes the script's output into its body. The Excel-fidelity reviewer can re-run it instead of redoing the sums.
  - Both report.json files must come from a binary with T-4159. An older one has no `closeness` on its cells, and
    the script exits 2 rather than guess.
  - report.json grows by the two new keys on each listed cell. Readers that ignore unknown keys, `rustsheet-eval`
    included, are unaffected.
