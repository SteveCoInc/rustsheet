---
id: T-5091
title: "SPEC-0037 and the catalog usage tier: Phase 1 tier gates and the envelope ratchet (decision 0355)"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A spec in docs/specs/ that says how decision 0355's tier gates and envelope ratchet are built. Agents can then cut it
into S/M implementation tasks, each one mergeable on its own.

## Context
- Decision 0355 replaces Gate A and Gate B with gates by usage tier (foundational and expert) over a 12-digit or
  1e-9-of-the-math envelope. It also changes the CI eval gate, from a bit-level ratchet to a per-cell envelope
  ratchet.
- The decision's numbers come from the report of main b84a7256 and docs/research/function-usage-2026-11-08 (usage.tsv
  lists every function's tier). The spec may use them as its worked example. It needs no new eval run.
- Code to read:
  - crates/rustsheet-eval/src/closeness.rs (`within_significant_digits`, the 13-digit family);
  - attribution.rs (decision 0118's attribution);
  - report.rs (`v15_checkpoint`, `functions`);
  - milestone_exit.rs;
  - the eval gate and baseline code (`make eval-check`, `make eval-baseline`);
  - scripts/phase1_exit.py and scripts/exit_owners.py;
  - eval/catalog/functions.yaml and `rustsheet-catalog-view`.

## Acceptance criteria
- [x] The spec covers each of these:
  - the `usage:` catalog field and how scope.json carries it;
  - the envelope check: one helper, reused, never a copy;
  - the report rows for each tier's gate, beside the old Gate A and Gate B numbers, which stay as information;
  - the milestone rows under decision 0355 item 4;
  - the `scripts/phase1_exit.py` lines;
  - the envelope ratchet in the eval gate and its "bit drift" line;
  - what eval/baseline keeps (exact values) and how `--allow-regressions` and decision 0329 fit;
  - the CLAUDE.md directive 2 wording.
- [x] The spec lists the implementation tasks it cuts: S/M, one lane each, in the order they land. Until the last
  one lands, the old gates stay in force.
- [x] `make check` green; reviewers approve.

## Out of scope
- Moving functions between tiers (that needs a decision record).
- Fixing any cell.

## Outcome
- Decision 0355 is accepted with the owner's rulings of 2026-11-08: the expert tier needs 99.9% pooled and 99% per
  function, on both type and envelope; the foundational/expert cut is prior ≥ 0.5 or ≥ 6 SpreadsheetBench
  workbooks; foundational tasks come first at equal priority (item 7); every report leads with the tiers (item 8).
- docs/specs/SPEC-0037-usage-tier-gates.md designs it; the rollout tasks are T-5092 to T-5095.
- SPEC-0037 design 1 is done here:
  - `usage: foundational` on 122 catalog entries (the header documents the field);
  - `rustsheet-catalog-view` writes `usage` on every scope.json row, `expert` by default;
  - `catalog::Usage` reads it in the harness;
  - tools/tests/test_catalog.py holds the catalog to usage.tsv, and test_catalog_view.py and catalog.rs test the
    view.
- `ENVELOPE_DIGITS` and `cell_envelope` are in accuracy/envelope.rs, with unit tests, so that T-5092 and T-5093 share one
  per-cell envelope test. Nothing calls it yet.
- docs/milestones.md states the Phase 1 exit by tier, and says the old rules hold until each rollout task lands.
- Eval delta: none (no gate reads `usage` yet).
