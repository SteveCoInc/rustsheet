---
id: T-4716
title: "Gate A aimed-stratum hygiene after T-4059: refuse aimed outside e-* batches, mark e-T-3141 as train data, T-3141 doc trims"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4059, T-3141]
---

## Goal
Clear the MINOR review findings that T-4059 (PR #3761) and T-3141 (PR #3105) deferred.

## Context
- tools/src/rustsheet_tools/accuracy/cli.py `generate`: SPEC-0029 allows `--stratum aimed` only in `e-T-XXXX`
  batches. The CLI does not enforce that, which is how e-T-3141 was first drawn under the batch name `r4-T-3141`.
- tools/src/rustsheet_tools/accuracy/model.py, the `Definition` docstring: it still calls an aimed batch "an `r4-*`
  sweep". It should say an `e-*` evidence batch.
- SPEC-0029 "The R4 guards" says the tables never feed a fit. accuracy/{CUMPRINC,CUMIPMT}/e-T-3141 is T-3141's fit
  data, and its case description names batch `r4-T-3141`: its seed comes from that name (`--batch r4-T-3141 --stratum
  aimed` reproduces it). Record both facts in the spec.
- docs/tasks/M9/T-4059-*.md acceptance criterion 2: its cells pass only once T-3141 lands. Say so in the box.
- crates/rustsheet-engine/src/test/test_fn_cumulative_lift.rs, `an_underflowed_principal_is_one_core_for_both_types`:
  one comment line runs past 100 columns. Rewrap it.
- crates/rustsheet-engine/src/functions/financial/annuity.rs, the `PAYMENT_EXPM1_TO` doc: trim the CUMPRINC evidence
  to a pointer at decision 0327 and financial.md.

## Acceptance criteria
- [ ] `rustsheet-accuracy generate X --stratum aimed --batch <name>` refuses any batch name that is not `e-T-NNNN`,
  and a unit test covers it.
- [ ] The model.py docstring, the SPEC-0029 note and the T-4059 acceptance box are corrected. The comment is rewrapped
  and the doc trimmed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-4710's open misses.
