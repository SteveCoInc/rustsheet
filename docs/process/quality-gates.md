# Quality gates

Every gate is automated and runs in CI (`.github/workflows/ci.yml`), on each PR head and again on
the merge group. `make check` runs the same gates locally, except `cargo deny` and `cargo machete`
(`make ci`). Before a push, workers run `make check-fast`: the gates their change can break, by
path, never the full Rust suite (`scripts/fast_gates.py`, decision 0248). Gates are **ratchets**: a number that may only improve. Lowering a bar requires a
decision record in `docs/decisions/` (enforced by `scripts/check_governance.py`).
On docs-only PRs (decision 0049) CI skips the rust, eval and python jobs; hygiene always runs, and
`docs/excel-semantics/` counts as code because the eval harness reads it. The rules are
`scripts/ci_changes.py`, run from the PR's base commit (decision 0066).

| Gate | Command | Rule |
|---|---|---|
| Formatting | `make fmt-check`, `ruff format --check` | rustfmt / ruff formatted |
| Rust lints | `make clippy` | workspace policy in `Cargo.toml` (`clippy::pedantic`, `unwrap_used`, `panic`, `print_*`, `unsafe_code = deny`), `-D warnings` |
| Lint debt | `scripts/check_lint_debt.py` | per-file `LINT-DEBT` `#![expect(...)]` blocks (inherited from the fork, decision 0335) may only shrink, and no new file or manifest may gain one; locked in `quality/lint-debt.json` |
| Rust tests | `make test` (CI: `test-shard-1`, `-2` and `-3`, decisions 0122 and 0232) | all pass |
| **Excel compatibility** | `make eval-check` (CI and `make check`: `make eval-gates`, decision 0125) | no regressions against `eval/baseline/*.txt` (known failures): no cell leaves decision 0355's envelope or gets a wrong type, error or text, while bit drift inside it is recorded, not blocked; baseline must be up to date. The gate suites include the fetched poi and corpus-bench, judged in CI's `eval (fetched gate suites)` job beside `eval (vendored suites)`. The `eval (Excel-compatibility gate)` summary job keeps the required name. CI restores them from a cache keyed on their pins, fetches them on a miss, and fails when one is still not fetched (`--require-fetched`). The local targets fetch them first when missing or stale (decision 0344, SPEC-0035). calipers is fetched but ungated (SPEC-0035 D4) |
| Round-trip fidelity | `rustsheet-eval roundtrip --check` (CI and `make check`: `make eval-gates`, decision 0307; nightly: decision 0323) | no fidelity item of a gated round-trip suite (`eval/roundtrip/`, SPEC-0033) regresses against `eval/baseline/roundtrip-*.txt`; a case may not compare less or stop running. CI's PR gate checks the vendored suites, `roundtrip-fixtures` and `roundtrip-excel-saved` (Excel-saved packages committed as evidence, T-4187), and `roundtrip-poi` and `roundtrip-corpus-bench` from its cached pinned fetch of those corpora (decision 0344, SPEC-0035). `make check` fetches them first when their cache is missing or stale. The nightly job fetches all three corpora and also checks Calipers (T-4632). Reproduce with `rustsheet-eval fetch --suite <suite>` and `rustsheet-eval roundtrip --check --timeout 300 --suite roundtrip-<suite>` |
| Visible parity | `make eval-check` (CI and `make check`: `make eval-gates`) | no case of the vendored gated suites may have more in-scope cells outside decision 0355's envelope than its line allows; `eval/baseline/visible-parity.txt` (decisions 0088, 0093, 0219, 0355) |
| Usage tiers | `make eval-check` (CI and `make check`: `make eval-gates`) | no usage tier (foundational, expert) recorded `met` in `eval/baseline/usage-gates.txt` may be unmet on the vendored gated suites (decision 0355 items 3 and 6) |
| Evidence floor | `make evidence-floor` | in-scope, oracle-evaluable functions with fewer than 20 Excel-evidence cells may not increase; `quality/evidence-floor.json` (decision 0062). CI and `make check` run it with `make eval-check` as `make eval-gates`, one run of the suites (decision 0125) |
| New-case owners | `scripts/new_case_owners.py` (CI and `make check`: `make eval-gates`) | every visibly-off Excel cell (decision 0188 R1(c)) of an eval case the branch adds or refills (its case file or `.oracle.json` sidecar added or changed against the merge base with `origin/main`) is owned by an open task by `scripts/exit_owners.py`'s rule; a `defer:` owner counts (T-3793, decision 0245) |
| Python tools | `make py-check` | ruff, mypy `--strict`, pytest |
| Python bindings | `make py-bindings` | maturin build + pytest |
| Duplication | `scripts/check_duplication.py` | duplicated lines (10-line normalized windows) may not increase; `quality/duplication.json` |
| Fitted constants | `scripts/check_fitted.py` | every empirically fitted constant, cut-point or table carries one `/// FITTED(T-NNNN, <n> cells, <range>)` doc line (`range unknown` when the evidence does not say); malformed tags and tags citing a task with no file fail; the count per source file may only fall, and a rise needs `--update` in a PR citing the fit task and its hold-out result; `quality/fitted.json` (decision 0188 R4.3) |
| File size | `scripts/check_file_size.py` | new source files ≤ 1000 lines; inherited oversized files may not grow; `quality/oversized-files.json` |
| Tasks | `scripts/tasks.py validate` | task schema, ids, dependencies, no unsplit L tasks; every `TODO(T-XXXX)` under crates/, bindings/, tools/, scripts/ names an open task (decision 0072); a task the branch adds lists no cell another open task lists unless one names the other (T-3229, decision 0182) |
| Dependencies | `cargo deny check` | license allow-list, RustSec advisories, sources, no wildcard versions |
| Unused deps | `cargo machete` | none |
| Spelling | `typos` | clean (Excel names allow-listed in `_typos.toml`) |
| Docs readers | `scripts/check_docs_readers.py` | code may read `docs/` or `.claude/` only under `CODE_READ_PATHS` in `scripts/ci_changes.py`, which CI never skips (decision 0066) |
| CI rights | `scripts/check_ci_workflow.py` | `.github/workflows/ci.yml` references no secret or job token and every `permissions` block is exactly `contents: read`; its tests pin `on.push.branches` to `main` alone (decisions 0147, 0252) |
| Math sidecars | `make eval-math-check` | every Gate A one-call cell of the probe suites has a math value or a recorded generator skip in its `*.math.json`, and no entry's `f` is stale; regenerate with `cd tools && uv run rustsheet-accuracy math --suite <suite>` (decision 0243) |
| Governance | `scripts/check_governance.py origin/main` | protected-path changes need a decision record; test deletions flagged |

## Ratchet mechanics

- **Eval baseline** (`eval/baseline/<suite>.txt`): records each case's checked-cell count and the
  exact known-failing cells. The ratchet is decision 0355's envelope, not each bit (item 6,
  SPEC-0037 design 6).
  - `F<TAB>case<TAB>item` lists every bit-level known failure (Phase 1b's to-do list), and
    `E<TAB>case<TAB>item` those of them outside the envelope: a cell `cell_envelope` calls
    `Outside` (a number off both 12 digits of Excel and 1e-9 of the math) or `WrongType` (a wrong
    type, error, text or boolean). Every `E` item is also an `F` item. A whole-case item (`*`,
    `!status`) is always `E`: a crashed or missing case is never drift. In a `cases`-mode suite
    (corpus-bench) a case fails `*` only when one of its cells is outside the envelope.
  - In a case whose checked-cell count is unchanged:
    - a **regression** is a new `E` item, including an item that drifted before and now leaves
      the envelope;
    - **drift** is a new `F` item without an `E` item: allowed, printed as `drift case Sheet!A1`;
    - an item that leaves `E` but stays in `F` moved **into the envelope**, recorded like a fix.

    Failing items in new or changed cases are new coverage, as before.
  - `make eval-check` fails on any regression, and while the file is out of date (a fix, a move
    into the envelope, drift, new coverage or a new case), so a PR's diff shows every drift line.
    Each suite's summary line counts `drift N` and `into envelope N`.
  - `make eval-baseline` rewrites it, recording fixes, moves into the envelope and drift, and
    refuses regressions unless `--allow-regressions` is passed. That flag is only legitimate with
    a decision record, for example when an oracle re-run shows the old expectation was wrong.
  - A baseline without the `E` kind in its header (written before decision 0355, and every
    round-trip baseline) is judged as if every `F` item were an `E` item.
  - Displayed text (a sidecar's `text` map, decision 0068) is gated the same way on its own lines,
    `TC` (a case's scored text cells) and `TF` (a known text failure), in every gated suite
    (decision 0070). So is the layout of a pivot a case compares in full (an engine-export
    case's): `LC` (its compared pivots) and `LF` (a pivot whose location or saved
    `rowItems`/`colItems` are not Excel's, `Sheet!Pivot`; T-4944).
- **Accepted deviations** (`eval/suites/<suite>/deviations`, decision 0036): cells that differ
  from Excel by design (a `Status: DEVIATION` entry, decision 0014), each with RustSheet's pinned
  by-design value.
  - A listed cell giving the pinned value scores as a *deviation*: the report's `deviations`
    column, outside the bit-exact denominator, never an F line. Any other wrong value is a
    failure. A listed cell matching Excel passes with a stale-line warning; remove that line.
  - `make eval-check` and `make eval` fail when a line is malformed, names a missing case or an
    unscored cell, or cites an entry without a `Status: DEVIATION` line.
  - Changing the file is a protected change (`scripts/check_governance.py` asks for a decision
    record): the DEVIATION entry lands in the same PR, and the
    Excel-fidelity reviewer confirms the limit is a directive-5 limit.
  - A difference from Excel is accepted only for directive 5 (a `DEVIATION` entry, decision 0014)
    or inside a `MATH-OVER-EXCEL` case listed in decision 0188 R3, or in a later decision record
    carrying an owner ruling (a new region needs one). `MATH-OVER-EXCEL` cells are not lines of
    this file: they stay F lines of eval/baseline, which Gate B and R1(c) count as passing when
    the engine returns the math within ε.
    Anything else is a CI failure (an F line in eval/baseline, or a regression the gate refuses).
- **Evidence floor** (`quality/evidence-floor.json`, decision 0062): `rustsheet-eval
  evidence-floor` runs the vendored suites (fetched external suites do not count) and counts the
  in-scope catalog functions the oracle can evaluate with fewer than 20 Excel-evidence cells. It
  fails when the count rises and when it falls without the lock being updated; `make
  eval-baseline` records a drop. Recording a rise needs `--update --allow-increase` and a
  decision record (for example, when the catalog brings functions into scope).
- **Visible parity** (`eval/baseline/visible-parity.txt`, decisions 0088, 0219 and 0355): a full
  `rustsheet-eval check` also counts, per case, the in-scope cells of the vendored gated suites
  outside decision 0355's envelope (`cell_envelope`'s `Outside` and `WrongType`; before decision
  0355 it counted the cells visibly off at 15 digits, decision 0083). A lock whose header still
  names the old measure fails `check` as out of date, and the next `make eval-baseline` lowers
  every case to its new count (a rise still needs `--allow-regressions`). The lock has one
  `case<TAB>count` line per case with
  any such cell, and fails each case above its line; a case with no line allows none. `make
  eval-baseline` records the line of a case new to its suite's baseline, so a PR that adds cases
  commits their lines. It never lowers a line, so fix PRs leave the file alone, and the steward
  lowers the lines daily with `make eval-lower-visible-lock` (decision 0093). Raising an existing
  case needs `--allow-regressions` and a decision record. PRs that touch different cases change
  different lines, so they merge cleanly. `make catch-up` resolves a conflict on the file three-way
  per case, then re-records with `NEW_CASES=<file>` (`update-baseline --new-cases`). That file lists
  the case files the PR adds, and only those cases may then get a new or higher line. No merge may
  leave one of `main`'s cases above `main`'s line (decision 0219).
- **Usage tiers** (`eval/baseline/usage-gates.txt`, decision 0355 items 3 and 6): a full
  `rustsheet-eval check` computes the two tier gates on the vendored gated suites' cells (each
  Excel cell toward its sole non-glue call, decision 0118, leaving out deviations, decision 0134
  exceptions, the pivot suites and GETPIVOTDATA): foundational needs 100% of its cells right on
  type and at least 99.99% inside the envelope; expert at least 99.9% of its cells on both and
  at least 99% of each function's cells on both. The lock holds `foundational<TAB>met|unmet` and
  the same for expert. `check` fails when a tier written `met` is unmet, and while the lock is out
  of date; `make eval-baseline` writes `met` when a tier becomes met, and `unmet` over `met` only
  with `--allow-regressions`. The full report's tiers (`make eval`) are the Phase 1 exit measure;
  this lock is the CI ratchet on the cells CI runs.
- **Lint debt / duplication / oversized files / fitted constants:** the scripts fail on increases. When you reduce
  them, run the script with `--update` and commit the JSON in the same PR.

## What reviewers check that tools cannot
See `.claude/agents/reviewer-*.md`:
- semantic duplication
- layering
- spec adherence
- Excel evidence
- panic-on-untrusted-input paths
- complexity and allocation
- test quality

## Planned additions

| Milestone | Gate |
|---|---|
| M1 | public-API docs coverage (`missing_docs` on the facade crate) |
| M3 | benchmark regression gate (> 5% slower fails), metamorphic incremental-vs-full suite |
| M4 | memory-per-cell budget test |
| M5 | coverage report (cargo-llvm-cov) with a ratchet on engine line coverage |
| M12 | `cargo semver-checks`, 24 h fuzzing (parser, xlsx importer) |
