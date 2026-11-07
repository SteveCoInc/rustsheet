---
id: T-4915
title: "Render pivotField@name and item@n captions on pivot labels"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine renders a pivot field's `name` attribute as its caption and an item's `n` attribute as its label, as Excel does.

## Context
Found by T-4132's first gen-pivot batch (excel-graph, 2026-11-07; seed 0, rustsheet-gen --milestone M16). Each case's description names its sheets' covering-array levels.

Excel shows a pivotField@name caption (`Product field`) where the field's header shows (gen-pivot/2251b0f5e5a256e1 P03!L7; the engine shows `Product`) and an item's `n` (`West shown`) on the item's label and in its subtotal label `West shown Total` (gen-pivot/01865529b5a09c0b P04!M10, gen-pivot/a6d74990f3ae755e P06!L8 and L11). The engine ignores both attributes (catalog PV-1113, PV-1081, both `web` unknown until now: Excel for the web opens and renders them).

The rule and its evidence: docs/excel-semantics/pivots.md "What the gen-pivot batch found (T-4132)". Code: crates/rustsheet-xlsx/src/import/pivots/table.rs (typing the attributes; another stream's code), crates/rustsheet-engine/src/pivot/layout/render.rs. The engine pivot code is another stream's: this task is that stream's to implement.

Split: one gen-pivot sheet can show several causes, so these spans can overlap those of the other T-4132 owner tasks (T-4807, T-4914, T-4916, T-4917, T-4918, T-4919, T-4920, T-4921, T-4922, T-4924, T-4925, T-4926, T-4927); each owns its own cause there.

Owner of these visibly-off cells (decision 0245), 72 values and texts on 19 sheets (a sheet's span is the bounding box of the cells this cause explains; other causes of the same sheet have their own tasks):
- gen-pivot/01865529b5a09c0b P04!M7:M142
- gen-pivot/01865529b5a09c0b P07!M7:P7
- gen-pivot/01865529b5a09c0b P12!M7
- gen-pivot/2251b0f5e5a256e1 P03!L7
- gen-pivot/2251b0f5e5a256e1 P07!N5
- gen-pivot/2251b0f5e5a256e1 P09!L7
- gen-pivot/2251b0f5e5a256e1 P11!L9
- gen-pivot/2251b0f5e5a256e1 P12!L9
- gen-pivot/36996978427905b2 P04!M7:M23
- gen-pivot/462f10551d480f9d P03!L6
- gen-pivot/462f10551d480f9d P05!L7:L23
- gen-pivot/640780d7b6f4ad53 P05!L8
- gen-pivot/640780d7b6f4ad53 P11!N5
- gen-pivot/a6d74990f3ae755e P06!L8:L11
- gen-pivot/ca5c8fa99c13d747 P08!L6
- gen-pivot/ec473260bb2065b3 P04!L7:L11
- gen-pivot/ec473260bb2065b3 P07!L7
- gen-pivot/f9e7edbb5b74011a P02!L5
- gen-pivot/f9e7edbb5b74011a P10!M5

Also owner of these pivot-corpus render cells (T-4906, Excel-saved files: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)"), 16 render cells of 7 fresh pivots: item `n` captions, a renamed manual group (`Other`, which the engine shows as `Group1`) among them:
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx AlternateDesktopDistroTable!A2
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx AlternateDesktopPlatformsTable!A5:A12
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx AlternateServerDistrosTable!A8
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx ServerPlatforms!A6:A13
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx AlternateServerPlatformsTable!A7:A12
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx HatedGUI!A8
- pivot-corpus-lo/xlsx/column-style-autofilter.xlsx MobileOS!A10

## Acceptance criteria
- [ ] The cells above pass (`rustsheet-eval run --suite gen-pivot`), and no other case regresses (`make eval-baseline`).
- [ ] The pivots.md entry says the engine now matches, with the implementing code path.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
