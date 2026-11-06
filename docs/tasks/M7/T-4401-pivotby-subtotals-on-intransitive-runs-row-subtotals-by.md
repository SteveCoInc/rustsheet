---
id: T-4401
title: "PIVOTBY column subtotals on intransitive runs: a class no record reached holds every record of the line"
milestone: M7
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-4388]
---

## Goal
PIVOTBY's column subtotals on an intransitive run hold what Excel's do, confirmed on a fresh hold-out: a column
subtotal whose class no record reached holds every record of the line (level 2 and deeper; T-4378's rule already
gives the reached classes).

## Context
- From T-4382 (probes/t4382-pivotby-intransitive-paths, excel-graph, 2026-11-05; docs/excel-semantics/coercion.md
  "PIVOTBY's other paths on intransitive runs (T-4382)"). Texts A = ab, B = ab&U+00AD, C = U+00AD&ab.
- Row subtotals already follow the rule since T-4359 (#3409, GROUPBY's subtotal lines): every row-subtotal cell
  of the probe passes on main (Calc!B16:C30).
- Level-2 column subtotals (three column fields, `col_total_depth` 3): a subtotal whose class no record reached
  holds every record of the line, as a leaf no record reaches does, where ours is `""`: over BxC BxB CyA AxB AxB
  (p q p q p) the (`ab`&U+00AD,y) subtotal is p `1, 3, 5`, q `2, 4`. `Grid::cell_rows`'s `Slot::Subtotal` arm.
- The rule was chosen on these cells, so it owes a hold-out probe (predictions committed before the fill), as
  T-4378 did; docs/research/t4382/gen_probe.py and predict_engine.py are the pattern.
- Cells: probes/t4382-pivotby-intransitive-paths Calc!B31, B32, B33, B35, B39, B41, C31, C33, C39 (level-2
  subtotals).
- Owned until PR #3430 (T-4388, the rule) lands, which passes them: probes/t4401-pivotby-unreached-subtotal-holdout
  Calc!B2, Calc!B3, Calc!B4, Calc!B5, Calc!B6, Calc!B7, Calc!B8, Calc!B9, Calc!B10, Calc!B12, Calc!B13, Calc!B14, Calc!B17, Calc!B18, Calc!B20, Calc!B21, Calc!B22, Calc!B23, Calc!C1, Calc!C3, Calc!C5, Calc!C9, Calc!C11, Calc!C13, Calc!C15, Calc!C16, Calc!C17, Calc!C19, Calc!C22, Calc!C24.

## Acceptance criteria
- [ ] Every probes/t4382-pivotby-intransitive-paths cell named in Context passes (or the cells the hold-out
      refutes are recorded in coercion.md and owned).
- [ ] A hold-out probe of fresh seeded runs (three and four column fields, both orders), written with the
      engine's predictions committed before the fill, passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Landing notes
- PR #3445 (draft), head 471a78207 before these notes. It waits on PR #3430 (T-4388, the rule), which was green and
  panel-approved but not queued when this session closed (00:3x UTC, 2026-11-06). Its owner session's landing
  window had closed, so it becomes an orphan after 4 h idle (pushed 23:10 UTC, 2026-11-05).
- Done: the hold-out probes/t4401-pivotby-unreached-subtotal-holdout (24 runs, 48 cells). Predictions were made on
  #3430's head e4c7b2248 and committed in d55b4eee6 before the fill. Score: 48 of 48, against 18 for main's rule.
  The 30 cells #3430 fixes are recorded in the baseline and owned in Context, so CI can pass. coercion.md is updated.
  T-4382's own T-4401 cells (Calc!B31 to C39) pass on #3430's head merged with main.
- Next, once #3430 has merged:
  1. Merge origin/main in. eval/baseline/probes.txt and visible-parity.txt will conflict; take main's, then run
     cargo build --release -p rustsheet_eval
scripts/eval_guard.sh target/release/rustsheet-eval update-baseline 
[Thread debugging using libthread_db enabled]
Using host libthread_db library "/lib/x86_64-linux-gnu/libthread_db.so.1".
suite                         cases     cells    passed deviations    pass%
fixtures                         25       348       348          0  100.00%
gen-m1                           15      3750      3750          0  100.00%
gen-m10                          20      1690      1690          0  100.00%
gen-m2                          105     26001     26001          0  100.00%
gen-m5                          201     59916     59913          0   99.99%
gen-m6                          161     37448     37448          0  100.00%
gen-m7                          244    101666    101666          0  100.00%
gen-m8a                          12      3577      3577          0  100.00%
gen-m8b                           9      3368      3368          0  100.00%
gen-m9                          344    110931    109162          0   98.41%
ironcalc-calc                   227     73300     73030          0   99.63%
  note: 1 cell(s) passed by decision 0042 (graph 0 vs kept subnormal; each has a FALSE (x)=0 companion)
probes                         2695    224736    219777         19   97.80%
  note: 229 cell(s) passed by decision 0042 (graph 0 vs kept subnormal; each has a FALSE (x)=0 companion)
smoke                             6        46        46          0  100.00%
milestone list m1-probes: 89/89 cases run, 1858 cells, 1858 bit-exact, 0 deviations, 0 known failures in 0 cases
probe milestone M0-M2 (decision 0051): 844 cases, 51725 cells, 51722 bit-exact, 3 deviations, 0 known failures
[fixtures] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m1] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m10] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m2] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m5] known failures 3 -> 3, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m6] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m7] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m8a] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m8b] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[gen-m9] known failures 1769 -> 1769, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[ironcalc-calc] known failures 270 -> 270, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[probes] known failures 4941 -> 4941, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
[smoke] known failures 0 -> 0, text 0 -> 0  (fixed 0, new-coverage failures 0, REGRESSIONS 0, new cases 0, removed cases 0)
visible parity: 1952 in-scope cells visibly off (0 in new cases), eval/baseline/visible-parity.txt allows 2041 in 234 cases
[Inferior 1 (process 10611) exited normally]
target/release/rustsheet-eval evidence-floor --update
evidence floor: 0 in-scope functions with fewer than 20 Excel cells (lock 0 at floor 20).
  2. Check that all 48 cells pass, and drop the "Owned until PR #3430" Context bullet.
  3. Set the task done with an Outcome (eval delta: 30 fixed in t4401).
  4. Delete these notes, run the panel (design, security, fidelity), mark the PR ready and enqueue.
- Do not repeat: the predictions are already committed, so never refill or regenerate them. The case must not be
  refilled.
