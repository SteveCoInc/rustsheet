# 0268 — Phase 1 exit verdict reads Gate A

- **Status:** accepted (2026-11-04, T-4081)
- **Context:**
  - Decision 0188 R1(a) makes "Gate A holds" a Phase 1 exit condition. SPEC-0029 ("Gate A holds") requires every
    in-scope numeric function to have a table or an `exempt.tsv` line, no known failure and no pending point.
  - `scripts/phase1_exit.py` (decision 0187) printed no Gate A line, so its verdict and the dashboard's exit line
    left out the condition that is furthest from met.
  - The dashboard reads Gate A from `eval/runs/accuracy.json`. Only `make check` (eval-gates) writes that file,
    and the steward's run is `make eval`, so docs/progress.md showed "Gate A: n/a".
  - Measured on 2026-11-04 at 92a5d396: 14,354 points in 94 tables, 0 failing, 118 known failures and 199 missing
    tables. Gate A passes `make check` but does not hold.
- **Decision:**
  - `make eval` also runs `rustsheet-eval accuracy`. A failing point does not fail `make eval`, because
    `make check` gates it. The old JSON is deleted first, so a crash leaves none.
  - `phase1_exit.py` gains a `gate A` line read from that JSON's `holds`, with the counts: points, inside, known
    failures, failing, list problems and missing tables. It reads `n/a` when the JSON is absent, unreadable or older
    than the report. Like every `n/a`, that fails the exit.
- **Consequences:**
  - The exit verdict and the dashboard show Gate A, with its missing-table count, on every steward run.
  - Nothing else changes: no gate, threshold or table. The Makefile and scripts/ edits are why this record exists
    (protected paths).
