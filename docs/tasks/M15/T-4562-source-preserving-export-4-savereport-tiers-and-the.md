---
id: T-4562
title: "Source-preserving export 4: SaveReport tiers and the round-trip dropped verdict (SPEC-0034 D4, SPEC-0033 D7)"
milestone: M15
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4183, T-4559, T-4560]
spec: docs/specs/SPEC-0034-xlsx-source-preserving-export.md
---

## Goal
`save_xlsx_with_report` returns the tier (`Verbatim`, `Patched`, `Regenerated { reason }`) and every drop. The
round-trip suite's `dropped` verdict reads it (SPEC-0034 D4, SPEC-0033 D7).

## Context
`save_preserving`'s returned `SaveTier` is what fills `SaveReport.tier`; the `Patched` tier exists once T-4560 lands.

SPEC-0033 D7 defines `SaveReport`, `Dropped` and `DropReason`, and SPEC-0034 D4 adds the tier. The facade's
`save_xlsx` and `to_xlsx_bytes` keep their signatures.

## Acceptance criteria
- [x] The facade gains `save_xlsx_with_report` and `to_xlsx_bytes_with_report`, with Python `save_xlsx(...,
      report=True)`.
- [x] Round-trip items named by the report with an accepted reason score `dropped`.
- [x] `make check` green; reviewers approve.

## Out of scope
- Stage 4 preservation under regeneration.

## Outcome
- **rustsheet-xlsx `export/report.rs`** defines `SaveReport { tier, dropped }`, `Dropped { item, reason }` and SPEC-0033
  D7's `DropReason`, including `StaleAfterEdit`. The facade re-exports them (one definition), and `save_preserving`
  returns the report.
  - The report is coarse until stage 5. A regenerated save names each source part reached by a relationship type the
    export does not model, unless T-4950's pivot drawings kept it, as `NotPreserved`. calcChain is `Policy`.
  - A patched save names the calcChain it dropped. A verbatim save drops nothing.
  - `Dropped.item` is the source part's name, which the suite resolves to its key (SPEC-0033 D7 now says so).
- **Facade.** `to_xlsx_bytes_with_report` and `save_xlsx_with_report` are added; `to_xlsx_bytes` and `save_xlsx` keep
  their signatures. Python gains `save_xlsx(path, report=True)`, which returns `(item, reason)` pairs (with an
  overload in the `.pyi`).
- **Round-trip.** A new `dropped` verdict passes with weight 0. `roundtrip/dropped.rs` rescores part items by the
  report: a lost part with an accepted reason (`Policy`, `ActiveContentInPlainFormat`) is `dropped`, and a reported
  part that was written is `changed` (D7.3). Scenario-explained reasons come with T-5004 and T-5005.
- **Tests.**
  - xlsx: `the_save_report_names_what_each_tier_left_out`.
  - Facade: `the_save_report_names_the_tier`.
  - Eval: `an_accepted_drop_passes_and_a_reported_kept_part_is_changed`.
  - Python: `test_save_report_lists_what_the_save_left_out`.
- **Baselines.** Rerecorded; 94 entries shrank and none grew. R1 kept rates: calipers 98.81% → 99.10%, corpus-bench
  99.06% → 99.17%, poi 98.89% → 99.30%. R0 is unchanged.
