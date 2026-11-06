---
id: T-4570
title: "XLOOKUP under a lifted lookup_value returns a range if_not_found as a reference element (implement the mixed-array spec)"
milestone: M6
lane: functions-lookup
status: todo
priority: P2
size: M
tier: opus
campaign: semantics
depends_on: [T-4569]
---

## Goal
`XLOOKUP` with an array lookup_value and a range if_not_found returns Excel's mixed array: a value per hit and the
range itself, as a reference, per miss. Every reader of probes/t4563-xlookup-not-found-ref-element sees what Excel
sees.

## Context
- Evidence: lookup.md "XLOOKUP's if_not_found and ADDRESS's sheet_text over an array" (T-4543, T-4563), with
  probes/t4543-xlookup-not-found-address-sheet-lift and probes/t4563-xlookup-not-found-ref-element (excel-graph).
- The representation and each reader's rule come from T-4569's spec. Today
  `xlookup.rs` `xlookup_if_not_found` gives `#VALUE!` for a range under a lifted lookup_value.
- probes/t4563 A49 and A53 (`CELL("address",..)` of a reference on Data) include Excel's workbook name in brackets,
  which is the oracle file's hashed name. Check how other CELL("address") cases score that prefix before matching it.

## Acceptance criteria
- [ ] The visibly-off cells named below pass, with no regression.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Owner of the visibly-off cells of probes/t4543-xlookup-not-found-address-sheet-lift (W45, W46) and of
probes/t4563-xlookup-not-found-ref-element (A1, A5, A9, A13:A14, A17, A21:A22, A29:A30, A33, A37, A41, A45, A49, A53, A73, A109, A113, T13, T17, T21, T29, T33, T45, T49, T53, T73, T109, T113, U13, U17, U21, U29, U33, U45, U49, U53, U73, U109, U113, V1, V5, V9, V13, V17, V21, V25:V26, V29, V33, V37, V41, V45, V49, V53, V73, V109, V113, W13, W17, W21, W29, W33, W45, W49, W53, W73, W97, W101:W102, W109, W113, X13, X17, X21, X29, X33, X45, X49, X53, X73, X109, X113, Y13, Y17, Y21, Y29, Y33, Y45, Y49, Y53, Y73, Y109, Y113, Z13, Z17, Z21, Z29, Z33, Z45, Z49, Z53, Z73, Z109, Z113): Excel's reference element and its readers, where ours is
`#VALUE!`.

## Out of scope
- An array (not a range) if_not_found, which takes its first element and matches today.
