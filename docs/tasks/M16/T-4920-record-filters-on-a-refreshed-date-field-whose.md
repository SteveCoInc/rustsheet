---
id: T-4920
title: "Record filters on a refreshed date field whose cache listed numbers"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A refreshed date field whose saved cache listed numbers keeps Excel's record filters: hidden items hide every date item the refresh appends, and a page field's selected item index selects the refreshed list's item at that index.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

The case builder caches date-formatted values as `<n>` items; Excel's refresh appends `<d>` items for them (pivots.md "Item casing when cache and source disagree, and date items"). With hidden items on that field, Excel shows no record at all: rowItems 0 in gen-pivot/a6d74990f3ae755e P02, P08 and P12, gen-pivot/ca5c8fa99c13d747 P04 and gen-pivot/01865529b5a09c0b P06 (page or row field When with one to three items hidden), while the engine shows the records of the visible items. With a page item selected, Excel shows another item: gen-pivot/462f10551d480f9d P01 selects 2958465 and Excel shows 365 at M3 (and its records), gen-pivot/640780d7b6f4ad53 P12 selects 1462 and Excel shows 367 at M2. Whether the rule is about appended items in general (a new text item beside hidden ones) is not probed.

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-engine/src/pivot/refresh.rs, crates/rustsheet-engine/src/pivot/filter.rs. The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4915, T-4916, T-4917, T-4918, T-4919, T-4921, T-4922, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 98 values and texts on 7 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/36996978427905b2 P07!L6:M8
- gen-pivot/462f10551d480f9d P01!L3:T18
- gen-pivot/640780d7b6f4ad53 P12!L2:P10
- gen-pivot/a6d74990f3ae755e P02!M5:N7
- gen-pivot/a6d74990f3ae755e P08!O6:P6
- gen-pivot/a6d74990f3ae755e P12!L6:M8
- gen-pivot/ca5c8fa99c13d747 P04!M5:O8

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
