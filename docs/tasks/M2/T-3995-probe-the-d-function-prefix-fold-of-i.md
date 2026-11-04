---
id: T-3995
title: "Probe the D-function prefix fold of İ, ϐ/β, ﬀ and other final forms under fold_lookup"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence settles how D-function criteria (plain prefix and wildcard) fold `İ`, `ϐ`/`β`, `ﬀ` and the Greek and
Cyrillic final forms other than `ς`, and criteria.md's UNVERIFIED line for them is resolved.

## Context
- Follow-up of T-3929 (excel-fidelity review MINOR). T-3929 moved the D-function prefix criteria from `fold_case`
  to `fold_lookup` (`result_starts_with_units`, functions/criteria.rs) on evidence for weightless characters,
  Deseret, `ϳ`/`Ͷ`, `ϴ`/`ϑ`/`θ`, `Ε`/`ϵ`, `צ`/`ץ` and Mtavruli (probes/t3929-wildcard-fold-dfunctions). criteria.md
  "Case beyond ASCII" still says the D-function fold of `İ` is UNVERIFIED (T-3180) and assumes `ϐ`/`β`, `ﬀ` and the
  other final forms follow the list; under the new fold that assumption is now `fold_lookup`'s classes.
- Probe with excel-probe, like probes/t3929-wildcard-fold-dfunctions (one database and criteria range per row,
  DCOUNTA): criteria `İ`, `i`, `ϐ`, `β`, `ﬀ`, `ff`, and the final forms `ץ`/`צ` siblings (`ך`/`כ`, `ם`/`מ`, `ן`/`נ`,
  `ף`/`פ`) as plain prefix and with `*`, against subjects starting with each partner; add COUNTIF and exact MATCH
  columns for comparison.

## Acceptance criteria
- [x] An excel-graph probe covers the keys above; criteria.md states the D-function fold for them with cells.
- [x] The engine matches the probe, or every off cell is owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- `=` and COUNTIF's non-wildcard equality (T-3904).

## Outcome
Probe `probes/t3995-dfunction-fold-final-forms` (excel-graph, 2026-11-04): D-function plain prefix and wildcard criteria
leave `İ`/`i`, `ϐ`/`β`, `ﬀ`/`ff` apart and fold the Hebrew final forms `ך ם ן ף` with `כ מ נ פ` (as `ץ`/`צ`).
The engine's lookup fold lacked those four classes; added to `lookup_fold.rs` CLASSES. criteria.md updated. Probe case
passes (209 cells). No other cells off.
