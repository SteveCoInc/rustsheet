---
id: T-5091
title: "SPEC: Phase 1 tier gates and the envelope ratchet (decision 0355)"
milestone: M0
lane: harness
status: todo
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
- [ ] The spec covers each of these:
  - the `usage:` catalog field and how scope.json carries it;
  - the envelope check: one helper, reused, never a copy;
  - the report rows for each tier's gate, beside the old Gate A and Gate B numbers, which stay as information;
  - the milestone rows under decision 0355 item 4;
  - the `scripts/phase1_exit.py` lines;
  - the envelope ratchet in the eval gate and its "bit drift" line;
  - what eval/baseline keeps (exact values) and how `--allow-regressions` and decision 0329 fit;
  - the CLAUDE.md directive 2 wording.
- [ ] The spec lists the implementation tasks it cuts: S/M, one lane each, in the order they land. Until the last
  one lands, the old gates stay in force.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving functions between tiers (that needs a decision record).
- Fixing any cell.
