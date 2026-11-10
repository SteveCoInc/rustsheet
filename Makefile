# RustSheet developer & agent entry points. `make help` lists targets.
# CI runs `make ci`. Agents MUST run `make check` before opening a PR.

CARGO ?= cargo
UV ?= uv
EVAL := target/release/rustsheet-eval
# Long eval runs go through the crash-evidence wrapper (decision 0041, T-1118).
EVAL_GUARDED := scripts/eval_guard.sh $(EVAL)
# SPEC-0035 D3, decision 0344: locally the eval targets fetch the gate suites (poi, corpus-bench) whose
# cache is missing or stale. CI sets EVAL_FETCH empty (its own cached fetch step is the only fetch) and
# EVAL_GATE_FLAGS=--require-fetched, so a gate suite it failed to fetch fails the run instead of skipping.
EVAL_FETCH ?= 1
EVAL_GATE_FLAGS ?=
# The pre-fetch's failure is ignored (`-`): offline, the gate suites skip as before and CI judges them.
EVAL_PREFETCH = $(if $(EVAL_FETCH),-$(EVAL_GUARDED) fetch --missing --gates)
PY := python3

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-18s %s\n", $$1, $$2}'

# ---------------------------------------------------------------- Rust
.PHONY: fmt fmt-check clippy test test-shard-1 test-shard-2 test-shard-3 build
fmt: ## Format Rust and Python code
	$(CARGO) fmt --all
	cd tools && $(UV) run ruff format src tests && $(UV) run ruff check --fix src tests

fmt-check: ## Check formatting (CI)
	$(CARGO) fmt --all -- --check

clippy: ## Lint Rust with -D warnings (workspace lint policy in Cargo.toml)
	$(CARGO) clippy --workspace --all-targets -- -D warnings

test: ## Run Rust tests
	$(CARGO) test --workspace --exclude rustsheet_python

# CI runs `make test` as three shards (decisions 0122 and 0232): shard 1 runs the tests whose names
# contain one of the TEST_SHARD_1 substrings, shard 2 the tests matching TEST_SHARD_2 but not
# TEST_SHARD_1, shard 3 every other test (`--skip` matches the same way), so together they run
# exactly the tests of `make test`, each once; a new test lands in shard 3 unless it matches.
# Memory-bound whole-grid tests slow each other down on one runner: keep test_fn_groupby in
# shard 1, apart from test_fn_pivotby and every test_tailed_array test, and split
# test_tailed_array between shards 2 and 3. Decision 0122 says when to rebalance.
TEST_SHARD_1 := test::test_fn_groupby text_functions::fn_regex test::test_multi_area_arguments dynamic_evaluation::test_range_reads functions::text:: test::lookup_and_reference test::test_collation test::test_formula_work test::statistical
TEST_SHARD_2 := test::test_fn_pivotby test_tailed_array::keyed test_tailed_array::index test_tailed_array::lookup test_tailed_array::small_and_large test::test_nested_groupby_cells
SHARD_TEST := $(CARGO) test --workspace --exclude rustsheet_python --

test-shard-1: ## CI: the tests matching TEST_SHARD_1 (a third of make test)
	$(SHARD_TEST) $(TEST_SHARD_1)

test-shard-2: ## CI: the tests matching TEST_SHARD_2 and not TEST_SHARD_1 (a third of make test)
	$(SHARD_TEST) $(TEST_SHARD_2) $(addprefix --skip ,$(TEST_SHARD_1))

test-shard-3: ## CI: every test shards 1 and 2 do not run (the last third of make test)
	$(SHARD_TEST) $(addprefix --skip ,$(TEST_SHARD_1) $(TEST_SHARD_2))

build: target-dir-guard ## Release build of the eval harness
	$(CARGO) build --release -p rustsheet_eval

# Every eval gate runs target/release/rustsheet-eval by path, so a target directory outside this
# worktree (a shared CARGO_TARGET_DIR, a `target` symlink) would build one tree and run another's
# binary: refuse it (T-3832; docs/process/agent-workflow.md, "Worktrees").
.PHONY: target-dir-guard
target-dir-guard: ## Refuse a Cargo target directory outside this worktree (make build, make check-fast)
	@$(PY) scripts/target_dir_guard.py $(CURDIR)

# ---------------------------------------------------------------- Eval (Excel compatibility)
.PHONY: eval eval-check eval-gates eval-fetched-gates eval-baseline eval-lower-visible-lock eval-show evidence-floor
eval: build ## Run all suites; writes eval/runs/latest/report.{md,json}, then Gate A's eval/runs/accuracy.json (decision 0268)
	$(PY) scripts/desktop_owned.py --out eval/runs/desktop-owned.tsv
	$(EVAL_GUARDED) run --desktop-owned $(if $(EVAL_FETCH),--fetch-missing)
	@# Gate A for the Phase 1 exit verdict and the dashboard (decision 0268). A failing point does not fail
	@# make eval (make check gates it); a crash leaves no JSON, which both read as not measured.
	rm -f eval/runs/accuracy.json
	$(EVAL_GUARDED) accuracy > eval/runs/latest/accuracy.txt 2>&1 || true
	@grep -m 1 '^accuracy:' eval/runs/latest/accuracy.txt || echo 'accuracy: no summary; see eval/runs/latest/accuracy.txt'
	@# Round-trip fidelity of eval/roundtrip (SPEC-0033, decision 0307): eval/runs/latest/roundtrip.{json,md}.
	$(EVAL_GUARDED) roundtrip $(if $(EVAL_FETCH),--fetch-missing)

eval-check: build ## Gate: fail on any regression vs eval/baseline
	$(EVAL_PREFETCH)
	$(EVAL_GUARDED) check $(EVAL_GATE_FLAGS)

eval-gates: build ## eval-check and evidence-floor from one run of the suites, then Gate A's accuracy check, the round-trip gate and new-case owners (CI, make check; T-2761, decisions 0193, 0245, 0307)
	$(EVAL_PREFETCH)
	$(EVAL_GUARDED) check --evidence-floor $(EVAL_GATE_FLAGS)
	$(EVAL_GUARDED) accuracy
	$(EVAL_GUARDED) roundtrip --check $(EVAL_GATE_FLAGS)
	@# Cases the branch adds or refills: every visibly-off cell needs an open owner (T-3793, decision 0245).
	$(PY) scripts/new_case_owners.py --eval $(EVAL)

eval-fetched-gates: build ## The fetched gate suites' cells and round trips: CI's eval (fetched gate suites) job (decision 0344; locally `make eval-gates` covers them)
	$(EVAL_PREFETCH)
	$(EVAL_GUARDED) check --suite poi --suite corpus-bench $(EVAL_GATE_FLAGS)
	$(EVAL_GUARDED) roundtrip --check --suite roundtrip-poi --suite roundtrip-corpus-bench $(EVAL_GATE_FLAGS)

eval-baseline: build ## Ratchet eval/baseline and quality/evidence-floor.json after fixes (refuses regressions); NEW_CASES=<file> lists the only cases new to the visible-parity lock (make catch-up, decision 0217)
	$(EVAL_PREFETCH)
	$(EVAL_GUARDED) update-baseline $(if $(NEW_CASES),--new-cases '$(NEW_CASES)')
	$(EVAL_GUARDED) roundtrip --update-baseline
	$(EVAL) evidence-floor --update

eval-lower-visible-lock: build ## Steward only, daily: also lower the cases of eval/baseline/visible-parity.txt (decisions 0093, 0217)
	$(EVAL_GUARDED) update-baseline --lower-visible-lock

evidence-floor: build ## Gate: no more in-scope functions below the evidence floor (T-0056; CI runs it in eval-gates)
	$(EVAL) evidence-floor

eval-show: build ## Show one case: make eval-show CASE=ironcalc-calc/FINANCIAL/IRR.xlsx
	$(EVAL) show $(CASE)

# ---------------------------------------------------------------- Python tooling
.PHONY: py-sync py-check py-bindings progress-report eval-math-check
py-sync:
	cd tools && $(UV) sync -q

# Decision 0280 (T-3714): the tests that shell out to rustsheet-eval use a dev build, not the
# fat-LTO release build `snapshot.eval_binary` makes on demand (about 3 min of CI). A release
# binary that already exists (`make check` builds one first) is used as is; a caller's
# RUSTSHEET_EVAL_BIN wins over both.
PY_EVAL_RELEASE = $(wildcard $(CURDIR)/target/release/rustsheet-eval)
PY_EVAL_BIN = $(or $(RUSTSHEET_EVAL_BIN),$(PY_EVAL_RELEASE),$(CURDIR)/target/debug/rustsheet-eval)

py-check: py-sync ## Lint, type-check and test tools/ (pytest on every core, decision 0233; dev rustsheet-eval, decision 0280)
	cd tools && $(UV) run ruff check src tests && $(UV) run ruff format --check src tests && $(UV) run mypy
	$(if $(RUSTSHEET_EVAL_BIN)$(PY_EVAL_RELEASE),,$(CARGO) build -q -p rustsheet_eval)
	cd tools && RUSTSHEET_EVAL_BIN="$(PY_EVAL_BIN)" $(UV) run pytest -q -n auto

eval-math-check: build py-sync ## The probe suites' math sidecars are current: every Gate A one-call cell has a math value or a recorded generator skip, and no entry is stale (T-3588, decision 0243)
	cd tools && $(UV) run rustsheet-accuracy math --check --suite probes --suite probes-numeric

progress-report: py-sync ## Regenerate docs/progress.md and docs/progress/ from eval/runs/latest/ (after make eval; SPEC-0022, decision 0120)
	mkdir -p eval/runs/latest
	$(PY) scripts/tasks.py stats --json > eval/runs/latest/tasks.json.tmp || { rm -f eval/runs/latest/tasks.json.tmp; exit 1; }
	mv eval/runs/latest/tasks.json.tmp eval/runs/latest/tasks.json
	$(PY) scripts/phase1_exit.py --json > eval/runs/latest/phase1-exit.json || [ $$? -eq 1 ] || rm -f eval/runs/latest/phase1-exit.json
	cd tools && $(UV) run rustsheet-progress-report --out-dir ../docs

py-bindings: ## Build the PyO3 bindings into bindings/python/.venv and run their tests
	cd bindings/python && $(UV) venv -q --allow-existing .venv && $(UV) pip install -q --python .venv maturin pytest
	cd bindings/python && . .venv/bin/activate && maturin develop -q && pytest -q tests

# ---------------------------------------------------------------- Hygiene ratchets
.PHONY: hygiene deny machete typos
hygiene: ## Lint-debt, file-size, duplication, fitted-constant, tasks and governance ratchets
	$(PY) scripts/check_lint_debt.py
	$(PY) scripts/check_file_size.py
	$(PY) scripts/check_duplication.py
	$(PY) scripts/check_fitted.py
	$(PY) scripts/tasks.py validate
	$(PY) scripts/check_decisions.py
	$(PY) scripts/check_docs_readers.py
	$(PY) scripts/check_ci_workflow.py
	@# The scripts' test modules run in parallel, one process per module, largest file first (decision 0241); xargs fails if any module fails.
	ls -S scripts/test_*.py | xargs -n1 basename | xargs -P "$$(getconf _NPROCESSORS_ONLN)" -I{} $(PY) -m unittest discover -s scripts -p {} -q

deny: ## License/advisory/source policy (needs cargo-deny)
	$(CARGO) deny check

machete: ## Unused dependencies (needs cargo-machete)
	cargo machete

typos: ## Spelling (needs typos-cli)
	typos

governance: ## Protected-path changes need a decision record
	$(PY) scripts/check_governance.py $(BASE)

# ---------------------------------------------------------------- Aggregates
.PHONY: check check-fast ci
check: fmt-check clippy test eval-gates eval-math-check py-check py-bindings hygiene typos ## Every gate CI runs on a PR (typos too: CI fails on it)

check-fast: target-dir-guard ## The local pre-push gate: the gates of `make check` the change can break, never the full test suite (scripts/fast_gates.py; BASE=origin/main)
	targets="$$($(PY) scripts/fast_gates.py --base $(or $(BASE),origin/main))" && $(MAKE) --no-print-directory $$targets

ci: check deny machete

# ---------------------------------------------------------------- Tasks
.PHONY: tasks next-task ready-tasks board new-task new-decision exit-owners unowned-off phase1-exit
tasks: ## List tasks (LANE=, STATUS=, MILESTONE= filters)
	$(PY) scripts/tasks.py list $(if $(LANE),--lane $(LANE)) $(if $(STATUS),--status $(STATUS)) $(if $(MILESTONE),--milestone $(MILESTONE))

next-task: ## Best ready task in a non-busy lane: make next-task LANE=functions [TIER=sonnet|opus] (LANE=functions = any functions-* lane)
	$(PY) scripts/tasks.py next $(if $(LANE),--lane $(LANE)) $(if $(TIER),--tier $(TIER))

ready-tasks: ## Every task next-task could pick, then "N ready": make ready-tasks TIER=sonnet [LANE=..] (decision 0021)
	$(PY) scripts/tasks.py ready $(if $(LANE),--lane $(LANE)) $(if $(TIER),--tier $(TIER))

new-task: ## New task file with a collision-free id: make new-task MILESTONE=M1 LANE=engine TITLE="..." [PRIORITY=P1] [SIZE=S] [TIER=opus] [USAGE=foundational|expert] (fetches origin; write a literal $ as $$)
	$(PY) scripts/tasks.py new --milestone "$$MILESTONE" --lane "$$LANE" --title "$$TITLE" $(if $(PRIORITY),--priority $(PRIORITY)) $(if $(SIZE),--size $(SIZE)) $(if $(TIER),--tier $(TIER)) $(if $(USAGE),--usage $(USAGE))

new-decision: ## New decision record with a collision-free number: make new-decision TITLE="..." (fetches origin; decision 0029)
	$(PY) scripts/new_decision.py --title "$$TITLE"

exit-owners: ## Failing exit cells of a milestone that no open task of it owns, after make eval: make exit-owners MILESTONE=M5 [STRICT=1] (decision 0096)
	$(if $(MILESTONE),,$(error MILESTONE=M<n> is required))
	$(PY) scripts/exit_owners.py --milestone '$(MILESTONE)' $(if $(STRICT),--strict)

unowned-off: ## Visibly-off cells no open task owns, after make eval: unattributed (decision 0132), or ALL=1 [FUNCTION=NAME] every one by function (T-0086)
	$(PY) scripts/exit_owners.py $(if $(ALL),--all-off $(if $(FUNCTION),--function $(FUNCTION)),--unattributed) $(if $(STRICT),--strict)

phase1-exit: ## One verdict per Phase 1 exit criterion after a full make eval; exit 1 unless every measured one is met (decision 0187) [JSON=1]
	$(PY) scripts/phase1_exit.py $(if $(JSON),--json)

board: ## Write docs/tasks/BOARD.md locally (git-ignored; decision 0017)
	$(PY) scripts/tasks.py board

# ---------------------------------------------------------------- Merge queue (decisions 0053, 0247)
.PHONY: merge-mode catch-up renumber
# Kept for the routines' stored prompts, which ask for `lock` before enqueueing (decision 0247):
# merges always go through GitHub's merge queue now, so it always prints `lock`.
merge-mode: ## Always `lock`, meaning "enqueue with enable_pr_auto_merge" (decision 0247; the merge train and its switch were deleted, T-3827)
	@echo lock

catch-up: ## Clear a merge-queue removal or a conflict with main that lies only in eval/baseline/ (decision 0207): make catch-up PR=123 merges main into the PR, re-records the lists and pushes; enqueue it again once CI is green (exit 0: pushed; 1: refused; 2: error)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_catchup.py --pr '$(PR)'

renumber: ## Clear an id collision with main (decision 0224): make renumber PR=123 gives the PR's colliding task and decision files fresh ids from the id registry, rewrites the old id only under docs/ in what the PR added (refusing when a line it added elsewhere names it), merges main in and pushes; enqueue it again once CI is green (exit 0: pushed; 1: refused; 2: error)
	$(if $(PR),,$(error PR=<number> is required))
	$(PY) scripts/merge_renumber.py --pr '$(PR)'
