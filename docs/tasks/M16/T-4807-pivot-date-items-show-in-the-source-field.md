---
id: T-4807
title: "Pivot date items show in the source field's date number format"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A pivot over a field of dates shows its date items in the source cells' date number format, as Excel does.

## Context
Found by T-2085: on probes/t2085-getpivotdata-items sheet D, rows When over date serials formatted m/d/yyyy,
Excel shows the items as 1/1/2024, 1/2/2024 and 2/1/2024, while the engine shows 45292, 45293 and 45323. The values
match; only the shown text differs. See pivots.md "Item casing when cache and source disagree, and date items" for
what is known about date items.

Owner of these visibly-off cells (decision 0245):
- probes/t2085-getpivotdata-items D!A4:A6

T-4132's gen-pivot batch shows the same on every pivot with the date field When on an axis or as a page field: the
date items' cells and texts (`12/31/2022` where the engine shows 44926), and the labels that hold an item, which are
text cells (`2/29/2024 Total` against `45351 Total`, `12/31/1900 Sum of Score` against `366 Sum of Score`), in the
source cells' format (m/d/yyyy, or m/d/yyyy h:mm with a time). Owner of these cells too (320 values and
texts on 50 sheets; a sheet's span is the bounding box of its date cells):
- gen-pivot/01865529b5a09c0b P02!P7:U7
- gen-pivot/01865529b5a09c0b P05!M7:O7
- gen-pivot/01865529b5a09c0b P07!L8:L12
- gen-pivot/01865529b5a09c0b P11!M6:N6
- gen-pivot/01865529b5a09c0b P12!N7:Q7
- gen-pivot/2251b0f5e5a256e1 P01!M6:M13
- gen-pivot/2251b0f5e5a256e1 P03!N6:Q6
- gen-pivot/2251b0f5e5a256e1 P04!O6:W6
- gen-pivot/2251b0f5e5a256e1 P05!N10:N19
- gen-pivot/2251b0f5e5a256e1 P08!L8
- gen-pivot/2251b0f5e5a256e1 P09!L8:L23
- gen-pivot/2251b0f5e5a256e1 P10!N8:N14
- gen-pivot/2251b0f5e5a256e1 P11!M7:AB7
- gen-pivot/36996978427905b2 P02!L9:L45
- gen-pivot/36996978427905b2 P03!L7:L21
- gen-pivot/36996978427905b2 P04!L7:L23
- gen-pivot/36996978427905b2 P06!N11:N21
- gen-pivot/36996978427905b2 P08!M7:AD7
- gen-pivot/462f10551d480f9d P03!N9:N42
- gen-pivot/462f10551d480f9d P04!M8:M79
- gen-pivot/462f10551d480f9d P05!M8:M42
- gen-pivot/462f10551d480f9d P06!O10:O34
- gen-pivot/462f10551d480f9d P07!M7:Q7
- gen-pivot/462f10551d480f9d P09!O7:R7
- gen-pivot/462f10551d480f9d P10!O6:S6
- gen-pivot/462f10551d480f9d P11!L11:L19
- gen-pivot/640780d7b6f4ad53 P04!O6
- gen-pivot/640780d7b6f4ad53 P06!M6:M13
- gen-pivot/640780d7b6f4ad53 P09!N9:N19
- gen-pivot/a6d74990f3ae755e P01!N8:N13
- gen-pivot/a6d74990f3ae755e P03!M8:M31
- gen-pivot/a6d74990f3ae755e P04!N7:AB7
- gen-pivot/ca5c8fa99c13d747 P02!L9:L18
- gen-pivot/ca5c8fa99c13d747 P05!M6:S6
- gen-pivot/ca5c8fa99c13d747 P08!M9:M11
- gen-pivot/ca5c8fa99c13d747 P12!M6:P6
- gen-pivot/ec473260bb2065b3 P01!L9:L35
- gen-pivot/ec473260bb2065b3 P04!N7:N21
- gen-pivot/ec473260bb2065b3 P05!M6:O6
- gen-pivot/ec473260bb2065b3 P07!M9:M13
- gen-pivot/ec473260bb2065b3 P08!M7:V7
- gen-pivot/ec473260bb2065b3 P09!N7:AZ7
- gen-pivot/ec473260bb2065b3 P11!P6:S6
- gen-pivot/ec473260bb2065b3 P12!L8:L16
- gen-pivot/f9e7edbb5b74011a P02!M6:M12
- gen-pivot/f9e7edbb5b74011a P03!O7:V7
- gen-pivot/f9e7edbb5b74011a P06!N10:N27
- gen-pivot/f9e7edbb5b74011a P08!L10:L17
- gen-pivot/f9e7edbb5b74011a P09!L7
- gen-pivot/f9e7edbb5b74011a P12!L8:L23

## Acceptance criteria
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] pivots.md "Item casing when cache and source disagree, and date items" says the engine now matches.
- [ ] `make check` green; reviewers approve.

## Out of scope
- GETPIVOTDATA with date items (T-4805).
