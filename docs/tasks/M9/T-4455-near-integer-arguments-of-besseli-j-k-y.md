---
id: T-4455
title: "Near-integer arguments of BESSELI/J/K/Y order, DEC2*/BIN2*/HEX2*/OCT2* places and number, and BITLSHIFT/BITRSHIFT shift: probe the 2^-22 snap against truncation, and match"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
The engineering functions' whole-number arguments read a value just below an integer the way
Excel does: the Bessel order, the base-conversion `places` and DEC2* number, and the BIT* shift
amount each follow the model their committed excel-graph cells show (LEFT's 2^-22 + 2^-33 snap,
DATE's asymmetric snap, or exact truncation). The engine today truncates all of them, which
the existing cells cannot tell apart from a snap.

## Context
- Evidence so far is all far from an integer, where truncation and the snap agree:
  - docs/excel-semantics/engineering.md: "A fractional order truncates: `BESSELJ(8,9.99)` equals
    `BESSELJ(8,9)`, `BESSELY(30,1.99)` equals `BESSELY(30,1)`"; "Any negative Bessel order is
    `#NUM!`, checked before truncation" with `BESSELK(5,0.9999)` = `BESSELK(5,0)` (probes/
    eng-gen-sample-edges E1:E10, T-1385); gen-m9/f8bc18a0e45a0e1a Calc!A61 `BESSELJ(7.999999,0.9999)`
    0.1716510411370656 (order 0).
  - DEC2* truncate the number toward zero and `places` truncates (`DEC2BIN(9,2.9)` `#NUM!`,
    `DEC2BIN(9,10.5)` "0000001001"; engineering.md "DEC2* truncate the number toward zero" and the
    T-1385 entry).
  - BIT* shift truncates (`BITLSHIFT(1,1.5)` 2, `(1,-1.5)` 0, `(1,53.5)` 0; engineering.md T-2421
    entry, probes/eng-besselk-underflow-bit-shift).
- One cell already shows that not every engineering argument snaps: the BIT* *number* with a
  fraction is `#NUM!` even 1E-10 below an integer, `BITLSHIFT(0.9999999999,-5)` is `#NUM!`
  (gen-m9/7c7e5bff12d93365 Calc!A145, excel-graph). Whether that is "no snap at all" or "a fraction
  check on the raw value before any snap" (as LEFT's negative check sees the raw value, text.md) is
  part of this task's question.
- The snap is real elsewhere: LEFT/RIGHT/REPLACE counts, INDEX, VLOOKUP/HLOOKUP, DATE, TIME,
  XMATCH/XLOOKUP modes and SORTBY sort_order (`near_integer_count` / `near_integer_whole` in
  crates/rustsheet-engine/src/functions/util/near_integer.rs; text.md "LEFT, RIGHT and REPLACE
  round counts up near an integer"). Sibling tasks: T-4453 (text), T-4454 (dates), T-4447 (SORT).
- Visible stakes: a snapped Bessel order changes the value completely (`BESSELJ(5,2-2^-23)` is J2
  or J1), a snapped `places` turns `#NUM!` into a padded result (`DEC2BIN(9,4-2^-23)`), and a
  snapped DEC2* number changes a digit (`DEC2BIN(2-2^-23)` "10" or "1"; `DEC2BIN(-1+2^-23)` "0"
  or "1111111111").
- Opus: no evidence exists near the cut, the probe has to separate three models per argument and
  the sign handling of DEC2* (truncation toward zero against an asymmetric snap) is a design
  question the cells must settle.

## Acceptance criteria
- [ ] A committed probe case (excel-probe skill, excel-graph sidecar) gives, for the Bessel order
      (all four functions), `places` (DEC2BIN, DEC2HEX, HEX2BIN, OCT2BIN at least), the DEC2*
      number (positive and negative) and the BITLSHIFT/BITRSHIFT shift (positive and negative):
      the values `n-2^-22-2^-33`, `n-2^-22-9*2^-36` and `n-1E-15` at two bases (one small, one at
      least 100 where the range allows), plus `-n+2^-23` and `-n+2^-22` for the signed arguments,
      and the BIT* number at `n-2^-22-2^-33` and `n-1E-15`.
- [ ] docs/excel-semantics/engineering.md records the model of every probed argument with its
      cells, next to the existing truncation entries.
- [ ] The engine reads each argument with its model through the existing near_integer helpers
      (no new tolerance constant); every probe cell passes bit-exact, and the cited existing cells
      (eng-gen-sample-edges E1:E10, gen-m9/7c7e5bff12d93365 Calc!A145,
      gen-m9/f8bc18a0e45a0e1a Calc!A61) keep passing.
- [ ] No new known-failure line in eval/baseline/; `make check` green; reviewers approve.

## Out of scope
- CONVERT, ERF/ERFC and the IM* functions (no whole-number arguments besides IMPOWER's power,
  which is real).
- BESSEL order caps and the BESSELI/BESSELK range edges (engineering.md "BESSEL order cap",
  T-2421, T-2550).
