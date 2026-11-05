---
id: T-4253
title: "Probe the labels Excel writes when it relabels a decimal numeric pivot group"
milestone: M6
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Evidence for the labels Excel writes when it relabels a numeric range group over decimal data, so the engine's
relabel (`NumberGroups::relabeled`) is VERIFIED or fixed.

## Context
- T-4235 (PR #3238) implements Excel's relabel of a `<groupItems>` list of the wrong length (probes/t4228-pivot-range-groups
  pivot 3: whole numbers, `0-3`, `3-6`, `6-9`, end groups first) as `General(start + k*interval)-General(start +
  (k+1)*interval)` in crates/rustsheet-engine/src/pivot/number_groups.rs `relabeled`. For decimal data (by 0.1, by
  0.001, from 1.1) the label format, and whether it is General, is UNVERIFIED: the probe's decimal labels already equal
  what Excel would write (docs/excel-semantics/web.md "Grouped pivot fields import as their group items").
- Probe with a wrong-text label list one item too long for decimal groupings, so Excel's relabel shows its own text;
  read the rendered labels (`=P!A4&""`) as probes/t4228 does. Use the `excel-probe` skill.
- Owner of the UNVERIFIED tags in `relabeled`'s doc and web.md that name T-4236 for decimal labels (T-4236 covers date
  groups only): retag them with this task.

## Acceptance criteria
- [ ] An excel-graph probe shows Excel's relabel labels for at least three decimal groupings; web.md records them.
- [ ] `relabeled` matches the probe, or its cells are owned by a task; the UNVERIFIED tags name this task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other wrong list lengths and partial last groups (T-4249).
