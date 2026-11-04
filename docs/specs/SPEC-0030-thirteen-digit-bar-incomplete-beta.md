# SPEC-0030 — The 13-digit bar for the incomplete-beta family

- **Status:** accepted (2026-11-04)
- **Milestone / lane:** M0 / harness
- **Tasks:** T-4055 (this spec), T-4056 (implementation), in one PR, as the owner asked (decision 0267)

## Problem
On probes-numeric at main a262d6b0, 89,170 attributed numbers are visibly off (decision 0083: more than half a unit
in the 15th significant digit from Excel). 84,932 of them are incomplete-beta family numbers that are within half a
unit in the 13th digit. In those regions Excel itself is rarely correctly rounded: about 1% of 2,026 sampled cells,
with a median error of 48 to 330 ulp (decision 0267). So the headline mostly measures how closely we reproduce
Excel's rounding noise. Decision 0267 moves these functions to a 13-digit bar.

## Goals / non-goals
- Goal: one definition of visibly off that every report, rate and exit reads, with the family at 13 digits.
- Goal: the cells this moves are counted and shown every run, in their own bucket, never dropped.
- Non-goal: changing what is bit-exact, or any baseline file.
- Non-goal: functions outside the family, such as T.DIST, GAMMA.DIST and the quantiles.

## Excel behavior to match
None changes. This is a counting rule. The evidence that the family's 14th and 15th digits are Excel noise is
decision 0267's Context and the docs/excel-semantics/statistics.md entries it cites.

## Design
- **The family.** `closeness::THIRTEEN_DIGIT_FAMILY` lists BETA.DIST, BETADIST, BINOM.DIST, BINOMDIST, F.DIST,
  F.DIST.RT, FDIST, NEGBINOM.DIST and NEGBINOMDIST.
- **Which cells qualify.** A cell's formula qualifies when every function it is blamed on (`closeness::blamed`:
  every non-glue function it calls) is in the family. An array member is judged by its anchor's formula. These keep
  the 15-digit bar:
  - a composite such as `BETA.INV(BETA.DIST(..))` or `F.INV(F.DIST(..))`;
  - a reference-only formula;
  - a cell with no formula.
- **The bucket.** `closeness::bucket`, the one function behind `Closeness::add`, `Visible::of` and `Visible::is_off`,
  gains `Residue13`. A `NumericClose` or `NumericMismatch` number lands there when three things hold: it is past
  `within_fifteen_digits`, it is within `within_significant_digits(.., 13)`, and its formula qualifies. That is the
  same half-unit rule at 13 digits, and `within_fifteen_digits` is now that function at 15.
- **Counting.** `Closeness` gains `residue13`, with `#[serde(default)]` so older report.json files still parse.
  - `scored()` includes it and `visible()` does not.
  - `Tier::of` places a function with residue cells no higher than `rate999`. `ulp4` and `digits15` still mean
    every cell is within 4 ulp or within 15 digits.
  - A wrong type, error or text is never residue.
- **Consumers.** Each one reads the bucket through `Visible` or `Closeness`, so each moves together with no code
  of its own:
  - report.json `closeness`, `attributed` and `excel`;
  - `attributed_off` and `unattributed_off`;
  - the Phase 1 rate and tiers (`v15_checkpoint`, `attribution`);
  - the probe rows and R1(c) (`r1c`, `probe_milestones`);
  - Gate B (`gate_b`) and the milestone exits (`milestone_exit`);
  - the visible-parity lock (`visible_parity`);
  - scripts/phase1_exit.py, exit_owners.py and new_case_owners.py, which read those report fields.
- **report.md.** The "Visibly off" table gains a `residue13` column.
- **Progress dashboard** (tools/src/rustsheet_tools/progress_report):
  - `model.BUCKETS` gains `residue13`, read as 0 from a report that predates it;
  - the bucket table, the closeness sentence and the charts show it;
  - history rows record `off_cells.residue13`, and "Since the last run" has a residue line, read as 0 for older rows.
- **What does not change.**
  - The known-failure baselines and `eval-check` regressions are bit-level.
  - The visible-parity lock still counts visibly-off cells, now at the family's bar. So a family cell that moves
    from the 15th to the 14th digit does not raise it. A move past the 13th digit still does.
  - Decision 0267 item 6 says when a bit-level regression of a family cell is allowed.

## Alternatives considered
- *Filter only in the dashboard.* Rejected: the Phase 1 rate, exits and tiers would disagree with the headline.
- *A per-function digit table for every function.* Rejected: decision 0267 rules only on this family, and a general
  table invites per-function loosening without evidence.
- *Count composites whose every call is in the family or its inverses.* Rejected: an inverse's search adds its own
  error, so it keeps the 15-digit bar until a decision rules on it.

## Test plan
- Unit tests (crates/rustsheet-eval/src/closeness.rs):
  - the 13-digit boundary;
  - family, `_xlfn.`-prefixed and glue-wrapped formulas;
  - non-family and mixed formulas;
  - array members;
  - type cells.
- tools tests: a closeness from before the bucket reads its residue as 0, and history rows carry it.
- `rustsheet-eval run --suite probes-numeric` at a262d6b0: attributed visible numbers 89,170 → 4,238, residue
  84,932. Type, error and bit-exact counts cannot move, because the bucket only takes numbers already past the
  15-digit bar.

## Rollout
One PR, with T-4055 and T-4056 together (decision 0267). After it merges, the steward's next `make progress-report`
shows the new headline. `--lower-visible-lock` then lowers the visible-parity lines the residue frees.
