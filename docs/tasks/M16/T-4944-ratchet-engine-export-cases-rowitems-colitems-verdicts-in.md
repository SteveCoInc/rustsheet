---
id: T-4944
title: "Ratchet engine-export cases' rowItems/colItems verdicts in the baseline"
milestone: M16
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An engine-export case whose engine `rowItems`/`colItems` are not Excel's fails in the baseline ratchet, as its cells do, instead of only being listed in report.md's "Pivot layout" section.

## Context
T-4177 compares the lines the engine saves (`pivot_scoring::saved_lines`) with the sidecar's, verdict `LinesDiffer`, but only reports it: the cells-mode baseline holds `Sheet!A1` items, and a line item would need a key that `exit_owners`, `new_case_owners` and the visible-parity lock read too. Today t4177-api-spill-refresh (Blocked) and t4177-spike (K, S) differ. Design the item key with T-4180, which extends this suite kind to the general excel-graph round-trip goldens (SPEC-0033 X3).

## Acceptance criteria
- [x] A lines mismatch of an export case is a baseline item, a new one a regression, with tests.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- **Key:** the case's baseline file gets two more line kinds, beside `TC`/`TF` (decision 0070's T-4944 consequences):
  - `LC<TAB>case<TAB>n`: the sidecar pivots a case compares in full (location and saved `rowItems`/`colItems`). The runner sets `CaseResult::layout_checked` for engine-export cases only.
  - `LF<TAB>case<TAB>Sheet!Pivot`: each of the case's `pivot_layout` entries, whatever its verdict. A `LocationDiffers` pivot hides its lines verdict, so a location fix that reveals wrong lines is not a new failure.
  - The `layout ` label prefix follows the ratchet's rules: a new `LF` in a case with an unchanged `LC` is a regression; one that clears is a fix; a new or changed `LC` is new coverage.
- **Scope:** other suites only report their layouts. `C`/`F` and every measure that counts `F` items are unchanged.
- **Reuse by T-4180:** its X3 goldens can fill the same lines for a saved package's pivots.
- **Owners:** `scripts/new_case_owners.py` reads `layout_items` (`case!Sheet!Pivot`, owned as a cell is, with the pivot name for the A1 name). `exit_owners.all_off_cells` leaves the items out explicitly. Tests for both.
- **Recorded `LF` items** (11, by `make eval-baseline`), with their owners:
  - T-4942: t4177-api-spill-refresh P!Blocked, t4177-api-spill P!Blocked and t4177-spike S!Blocked.
  - T-4941: t4177-date-flags-needed's DC, DO, DT, DS, DY, DM and DX (all LocationDiffers).
  - T-5050 (new): t4177-spike K!CalcTopSort, where 2 of 7 rowItems index Product's item A as 2 in Excel and 0 in the engine.

