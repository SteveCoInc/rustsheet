---
id: T-4686
title: "IMAGE option combinations, STOCKHISTORY of an empty cell and FIELDVALUE's row-axis broadcast that information.md leaves UNVERIFIED: probe and match"
milestone: M9
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells cover the IMAGE, STOCKHISTORY and FIELDVALUE argument forms that docs/excel-semantics/information.md
lists as UNVERIFIED. The engine matches each probe cell, and information.md states the observed results in place of
the UNVERIFIED lists.

## Context
- Gap triage 2026-11-06 (evening steward run, `make eval` on agent/steward/2026-11-06). The functions-logic lane
  (functions/logical/ and functions/information/) had no ready task. Every failing cell of the run in it has an open
  owner (T-4373 for CELL's desktop-only info types, T-4569/T-4570, T-4475/T-4476 for the LAMBDA helpers), so the
  lane's UNVERIFIED notes are the gap left. IMAGE, STOCKHISTORY and FIELDVALUE are not in eval/catalog/functions.yaml,
  so they rank as M9, prior 0.
- The UNVERIFIED items, all in information.md "IMAGE and STOCKHISTORY never fetch" (probes/rich-errors-image-sizing,
  T-1345; probes/rich-errors-arg-kinds, T-1141):
  1. IMAGE option validation: a `width` without a `height` (`0,,10`, `3,,10`), a negative `height` under `sizing` 2,
     `sizing` 2 with a `height`, `3,0` (height 0, no width), a fractional `sizing` near the limits (-0.5, 3.5), and an
     empty cell or numeric text as `sizing` or `height`.
  2. IMAGE: error order other than `source` before `alt_text` (`IMAGE(1/0,NA())` is `#DIV/0!`, the only order probed).
  3. STOCKHISTORY: an empty-cell `stock` (the engine gives `#CONNECT!`).
  4. FIELDVALUE: the row-axis broadcast `FIELDVALUE({1;2;3},{"a";"b"})`. The entry calls it UNVERIFIED and gives no
     value. The other three shapes are VERIFIED.
- What the engine does today (scratch evaluation, url `"https://example.invalid/a.png"`, ERROR.TYPE wrapped: 11 is
  `#BLOCKED!`, 3 `#VALUE!`, 10 `#CONNECT!`):
  - `IMAGE(url,"alt",0,,10)` 3, `(url,"alt",3,,10)` 3, `(url,"alt",2,-1)` 3, `(url,"alt",2,10)` 11, `(url,"alt",3,0)` 3,
    `(url,"alt",-0.5)` 3, `(url,"alt",3.5,10,10)` 11, `(url,"alt",Z1)` with Z1 blank 11, `(url,"alt","1")` 3, and
    `(url,"alt",3,N1,N1)` with N1 the text "10" 3.
  - `IMAGE(url,NA(),1/0)` 7 and `IMAGE(NA(),1/0)` 7, i.e. the first error in argument order.
  - `STOCKHISTORY(Z1,DATE(2020,1,2))` with Z1 blank 10.
  - `ROWS(FIELDVALUE({1;2;3},{"a";"b"}))` 3.
- Raw IMAGE cells come back from the oracle with an undecodable rich error type (information.md). Wrap every IMAGE and
  STOCKHISTORY call in `ERROR.TYPE` and read FIELDVALUE shapes through `ROWS`, `COLUMNS` and `ERROR.TYPE(INDEX(..,r,c))`
  for each element.
- Code: `fn_image`, `fn_stockhistory` and `fn_fieldvalue` in
  crates/rustsheet-engine/src/functions/information/offline_errors.rs.

## Acceptance criteria
- [x] probes/t4686-rich-value-option-residue.case.json is committed with excel-graph oracle values and holds every
      form of items 1-4. Item 2 covers at least `IMAGE(url,NA(),1/0)`, `IMAGE(url,"alt",NA(),1/0)` and
      `IMAGE(url,1/0,"x")`.
- [x] Every probe cell passes, or each cell that does not is listed in a new functions-logic task (`campaign:
      semantics` with a `## Visible-parity stage` section) filed with `make new-task` that names the case and its
      cells. Those new known failures are recorded with `make eval-baseline`.
- [x] information.md replaces each UNVERIFIED item above with the observed value and cites the probe cells. The
      "stored form" UNVERIFIED notes (undecodable rich values) stay.
- [x] Any engine change has a unit test in offline_errors.rs that pins the probed result.
- [x] `make check-fast` green; no new known-failure line in eval/baseline/ other than those owned as above; reviewers
      approve.

## Out of scope
- The fetch itself (a DEVIATION, decision 0014) and the stored rich-value form of IMAGE's error cells.
- STOCKHISTORY's other arguments beyond the empty-cell `stock`, and IMAGE's URL classification (T-1189's bullets).

## Outcome
Probe case `probes/t4686-rich-value-option-residue` (44 cells, excel-graph 2026-11-06) holds every form of items 1-4
and passes in full. Nine of the first 38 cells differed from the engine (and the percent and currency text cells added after the fidelity review), all fixed in `offline_errors.rs`:
- IMAGE `sizing` truncates toward zero before the 0..3 check (-0.5, -0.9 are valid) and text that ordinary number
  coercion reads (`"1"`, `"10%"`, `"$10"`) is that number for `sizing`, `height` and `width` (`Model::cast_number`).
- `sizing` 0, 1 and 2 all refuse a `height` and a `width` (2 was unchecked); `sizing` 3 needs a nonzero `height` or
  `width` (a width alone is enough).
- STOCKHISTORY of an empty-cell `stock` or `start_date` is `#VALUE!`, not `#CONNECT!`.
Items 2 (error order) and 4 (FIELDVALUE row-axis, 3x1 of `#FIELD!`, `#FIELD!`, `#N/A`) already matched. Unit tests in
`test_fn_offline_errors.rs` pin the probed results; `information.md` replaces the UNVERIFIED lists with the observed
values ("STOCKHISTORY and IMAGE residue (T-4686)"). Still UNVERIFIED and noted there: an empty-cell `height`/`width`
under `sizing` 0..2, date text as an IMAGE number, and an empty `end_date`, array element or array `stock` with an empty `start_date` in STOCKHISTORY.
No new known failures; the baseline gains the 44 passing cells.
