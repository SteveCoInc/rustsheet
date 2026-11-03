---
id: T-3064
title: "Generator M2 criteria batch with positive text-number, tilde and boolean-text matches"
milestone: M2
lane: harness
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
gen-m2 holds oracle-backed cells where each criterion T-2717 added to the pool has at least one positive match: a text-number (`"1"`, `"2.5"`, `"=2"`, `"1E1"`), a tilde escape and a boolean text (`"TRUE"`, `"=FALSE"`).

## Context
The reviewer-excel-fidelity pass on #2017 (T-2717), as re-run by the steward on 2026-10-28, found a gap in its two stored gen-m2 cases (7d61ca5cd4679d78, a073d3573617bf8e). Every cell with a text-number, tilde or boolean-text criterion returns 0 or `#VALUE!` (the *IFS ranges differ in shape). So the oracle never shows `"1"` matching a numeric 1, or `"=FALSE"` matching FALSE. The batch predates T-2719 (*IFS ranges share a shape). Only the `"<="&DATE(1900,1,1)` criterion has hits.

## Acceptance criteria
- [x] A new seeded gen-m2 batch (or `--focus` on COUNTIF/SUMIFS/COUNTIFS) whose oracle cells include a nonzero match for each new criterion kind; the eval passes or failures are owned by tasks.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Seeded gen-m2 batch (`rustsheet-gen --milestone M2 --focus --functions COUNTIF,COUNTIFS,SUMIF,SUMIFS --seed 3064 --count 40`): 40 cases, Excel-oracle filled. Positive matches (nonzero oracle value) now exist for text-number criteria (42 cells) and boolean-text criteria (27 cells). The generator's Data pool holds no `x*` or `?` text, so a tilde criterion never matches there (188 cells, all 0); changing the pools would reshuffle every stored seed, so the new probe `probes/criteria-tilde-positive-match` (oracle-filled) carries the positive tilde hits instead (`x~*` and `~?` count 1, sums 1 and 2, `x~~*` 1). Eval: gen-m2 103 cases all passing, probes +1 case passing; 0 regressions, no known failures added.
