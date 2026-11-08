---
id: T-5093
title: "Envelope ratchet in the CI eval gate: E lines, drift, the visible lock and the tier lock (SPEC-0037 step 3)"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-5091]
spec: docs/specs/SPEC-0037-usage-tier-gates.md
---

## Goal
The CI eval gate allows bit drift inside decision 0355's envelope and blocks a cell leaving it, a type or error
going wrong, and a tier gate going from met to unmet.

## Context
- SPEC-0037, design section 6.
- Code: crates/rustsheet-eval/src/baseline.rs (`Baseline`, `diff`, `Delta`), visible_parity.rs, main.rs
  (`cmd_check`), and the accuracy check; CLAUDE.md prime directive 2; docs/process/quality-gates.md.
- `cell_envelope` (accuracy/envelope.rs, landed with T-5091) is the only per-cell envelope test: call it, never
  write a second one.

## Acceptance criteria
- [x] Baselines carry `E` lines. `diff` reports regressions (new `E`), drift (new `F` without `E`) and moves into
  the envelope, and the check prints `drift N` and `into envelope N` per suite.
- [x] `update-baseline` records drift without `--allow-regressions`, and still refuses new `E` items in
  unchanged cases.
- [x] The visible-parity lock counts `Outside` and `WrongType` cells.
- [x] eval/baseline/usage-gates.txt holds each tier's met state, and the check fails on met → unmet.
- [x] The migration `make eval-baseline` run is committed: tool output only.
- [x] CLAUDE.md directive 2 and docs/process/quality-gates.md read as SPEC-0037 says.
- [x] Unit tests for every new `diff` kind, the lock change and the tier lock.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The tier gate computation itself (T-5092), beyond calling it.

## Outcome
- **Baselines** (`baseline.rs`): `Baseline` gains `outside` (`E<TAB>case<TAB>item`) and `envelope` (the header
  names the `E` kind). A failing cell is an `E` item when `cell_envelope` says `Outside` or `WrongType`
  (`baseline::outside_envelope`); whole-case items are always `E`; a `cases`-mode case fails `*` only with a cell
  outside. `diff` reports regressions (new `E`), `drift` (new `F` without `E`) and `into_envelope` (leaves `E`,
  stays `F`); both make the baseline dirty, neither needs `--allow-regressions`. A baseline without the `E` kind
  (written before this change, and every round-trip baseline) counts every `F` item as `E`, so the migration
  refused what the old gate refused and the round-trip ratchet is unchanged. Text and layout stay exact. The
  per-suite line prints `N -> M outside the envelope`, `into envelope N` and `drift N`, plus up to 25
  `drift case Sheet!A1` lines.
- **Visible lock** (`visible_parity.rs`): counts `outside_envelope` cells. A lock whose header predates decision
  0355 fails `check` as out of date, and an update re-measures it (lowers every case; a rise still needs
  `--allow-regressions`).
- **Tier lock** (new `gate_tiers.rs`): `function_cells` (Excel, scored, non-deviation cells of probes,
  probes-numeric and Gate B's suites, less pivot suites, GETPIVOTDATA and decision 0134 exceptions, each toward
  its `sole_call` when every call is in scope) and `tiers` (decision 0355 item 3's bars, integer rates via
  `meets_rate`) compute both tiers; eval/baseline/usage-gates.txt holds `tier<TAB>met|unmet`. A full gated
  `check` computes them on the vendored gated suites and fails on met → unmet or an out-of-date lock;
  `update-baseline` writes met, and unmet over met only with `--allow-regressions`. T-5092's `usage_gates`
  can build report.json on `function_cells` and `tiers` (it does not yet; the evidence-floor fallback of
  decision 0118 is not applied here: sole-call cells only).
- **Docs**: CLAUDE.md directive 2 (SPEC-0037 wording), docs/process/quality-gates.md (gate rows, the `E`/drift/
  into-envelope mechanics, the visible lock's measure, the tier lock), docs/architecture/eval-harness.md, and a
  note under decision 0355 item 6.
- **Tests**: `baseline` (drift, regressions incl. drift turning outside, into-envelope, changed cases, legacy
  baselines, `E` items per cell class, whole-case items in both modes, `cases` mode, the header round trip),
  `visible_parity` (counting `cell_envelope`, re-measuring an old lock), `gate_tiers` (cells counted, the
  foundational and expert bars at exactly 99%, 99.9% and 99.99%, check and update on the lock, parsing, the
  committed lock, the gate on disk). `cargo test -p rustsheet_eval`: 489 + 4 pass.
- **Migration** (`make eval-baseline` after merging main at 3c7190bec; tool output only): no regression, no
  drift. `F` lines unchanged in every cell suite; `E` lines: probes 1195 of 10262, gen-pivot 3063 of 3077,
  gen-m9 368 of 1714, pivot-export 163, poi 84, gen-m7 26, ironcalc-calc 4 of 265, gen-m5 0 of 3: 4903 in all.
  Into the envelope: 10691 items (probes 9067, gen-m9 1346, ironcalc-calc 261, gen-pivot 14, gen-m5 3).
  corpus-bench's one drift-only `*` line was dropped (fixed). The visible lock went from 7169 cells in 316 cases
  to 4818 in 183, no rise. usage-gates.txt: foundational unmet (142228 cells, 29 wrong type, 1 outside; TEXT,
  EDATE), expert unmet (276969 cells, 799 wrong type, 126 outside; IRR, RATE, BETA.INV, T.TEST, F.INV.RT,
  ACCRINT, MROUND, XIRR, F.INV). Round-trip baselines and quality/evidence-floor.json unchanged. `make
  eval-check` passes.
- **Follow-ups**: scripts/merge_catchup.py's resolvable lists do not include eval/baseline/usage-gates.txt, so a
  catch-up refuses a conflict there (rare: it changes only on a met/unmet flip).
