---
id: T-4253
title: "Probe the labels Excel writes when it relabels a decimal numeric pivot group"
milestone: M6
lane: engine
status: done
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
- [x] An excel-graph probe shows Excel's relabel labels for at least three decimal groupings; web.md records them.
- [x] `relabeled` matches the probe, or its cells are owned by a task; the UNVERIFIED tags name this task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other wrong list lengths and partial last groups (T-4249).

## Outcome
- Probes (excel-graph, 2026-11-05): probes/t4253-pivot-decimal-relabel (eight decimal groupings, by 0.1, 0.1 from 1.1,
  0.001, 0.25 from -1, 1e-5, 0.3, 1/3 and 12345678901.2, each with 13 wrong-text items for 12 or fewer layout items)
  and probes/t4253-pivot-relabel-order (eight lists for 0 to 1 by 0.1 with chosen items equal to Excel's labels).
- Findings (web.md "Grouped pivot fields"): a relabel bound is the 15-digit start + k * interval in plain decimal,
  not General (`0.00001`, not `1E-05`); the relabeled items are the file's items that are Excel's labels, in file
  order, then Excel's other labels in layout order. The "end groups first" of T-4235 was pivot 3's instance of this.
- Engine: `NumberGroups` keeps the relabel's layout-to-item map (`relabeled`) instead of `ends_first`, `new` takes the
  file's items, `labels` writes plain-decimal bounds (`label_number`), `relabeled_items` gives the cache items.
- Not a fit: the order rule is read off the probe (every point a distinct check), no constant chosen by scoring.
- UNVERIFIED, owned by T-4249 (amended): case and spacing of the label match, bounds of 1e15 or more or below 1e-5.
- Eval delta: new cases t4253-pivot-decimal-relabel 196/196 and t4253-pivot-relabel-order 224/224; no known
  failure changes, no regressions.
