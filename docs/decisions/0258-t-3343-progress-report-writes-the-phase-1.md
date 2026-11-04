# 0258 — T-3343: progress-report writes the Phase 1 exit JSON the dashboard reads

- **Status:** accepted (2026-11-03, T-3343). It applies decision 0187 and adds no rule.
- **Context:** The progress dashboard's goals block showed only `phase1_met` (the function-rate criterion). T-3343
  shows the overall Phase 1 exit verdict and the failing criteria, which `scripts/phase1_exit.py --json` already
  computes (decision 0187). The dashboard generator (tools/, `rustsheet-progress-report`) reads its inputs as files
  under `eval/runs/latest/` that `make progress-report` writes first, as it does for `tasks.json`, so the new input
  needs one line in the `progress-report` recipe of the Makefile, a protected path (`scripts/check_governance.py`).
- **Decision:** `make progress-report` runs `scripts/phase1_exit.py --json` into `eval/runs/latest/phase1-exit.json`
  before the generator. Exit 1 (Phase 1 not met) is a verdict, not a failure, so the file is kept; any other
  failure removes the file and the dashboard says "not measured" instead of stopping the run.
- **Consequences:** No gate, criterion or measure changes; `make check` does not run the recipe. The dashboard
  displays the verdict `phase1_exit.py` gives and computes none itself, so the two cannot disagree.
