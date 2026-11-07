# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-11-07, 05:53 UTC run (M6 is current)

This run follows #3746, the 2026-11-06 19:43 UTC run, squash-merged as 745c6e4c. It covers the 134 commits merged to `main` after ebe81baf (#3719, where that run's list stopped) up to 42d9bdaa (#3861), the head when this run started. `make eval` ran on 42d9bdaa. The tree held only this run's new task files, which no case reads.

- All 17 suites were scored. calipers, corpus-bench and poi were not in this container's cache. A first `make eval` skipped them, so I ran `rustsheet-eval fetch` for the three and ran `make eval` again; those are the numbers used here. Timeouts: 0 in every suite.
- Eval gate: `make eval-lower-visible-lock` shows 0 regressions and 0 fixed in every gated suite. It lowered the visible-parity lock (decision 0093) from 9,757 allowed in 299 cases to 3,102 in-scope visibly-off cells in 266 cases (0 in new cases).
- **Round-trip regression on `main`.** With the corpora fetched, `rustsheet-eval roundtrip --check` reports `REGRESSION roundtrip-poi/54436.xlsx * 296`. That case was 100.00% kept with no failing line in the baseline. I bisected with `roundtrip --suite roundtrip-poi --filter 54436`: e7f6fbb0 (T-4363) keeps 100.00% and cd0c355a (T-4176, #3800, pivot cache export) keeps 78.79%. The save now regenerates the whole package for this one-pivot workbook instead of keeping the source parts. I filed P0 T-4798 (engine, M16) and recorded nothing in the baseline. #3800 merged after SPEC-0035's PR gate (#3779), so T-4798 also checks whether that gate runs round-trip poi on PRs. The next Nightly eval should go red on this case until T-4798 lands. roundtrip-calipers, -corpus-bench and -fixtures are unchanged.
- The dashboard has this run's numbers (`make progress-report`; the judgment blocks pass `--check`).

CI on `main` is green: of the last 40 `ci.yml` push runs on `main`, 39 succeeded and the newest (42d9bdaa) was still running. The latest scheduled Nightly eval (2026-11-06 14:16 UTC, 7811739) and Nightly fuzz (2026-11-06 11:26 UTC, f90f8172) runs both succeeded. Neither has run since #3800.

Milestones: the current milestone stays M6 (decision 0027 order), and its exit fails only on its probes.
- M6's probes have 34 visibly-off cells. Five are t4697-large-small-median-mixed-sheet-refs, which #3847 (T-4714, queued this run) fixes. The other 29 are t4704-mixed-xlookup-grid-readers, which T-4704 probed and handed to its fix tasks T-4739 to T-4742.
- M7's exit row, met last run, is open again on 28 probe cells. These are the 2026-11-06 refills that decision 0329 makes the target for nested GROUPBY/PIVOTBY cells, owned by T-4736 (draft #3859), T-4737 and T-4738. M8 has 2 cells and M11 has 1, both on new probes.
- `make exit-owners MILESTONE=M6` reports 0, as do M7, M10 and M11. M8 lists t4618-order-after-active-backward Q!A1: T-3610 (an M9 harness task) names it, per T-4642, but no M8 task does. `MILESTONE=M9` lists 2 cells owned by tasks of other milestones: the merged-range fixture COUNTBLANK cell (T-4722, M11) and t2475-stat-subexpressions-refill-1106 P!A64 (T-4737, M7). `make unowned-off` found gen-m9/17c110a53924beb8 Calc!A300 with no open owner. This PR names it in T-4737, and the command now reports 0.
- The frozen M1 list has 0 known failures (89/89 cases), and the M0–M2 probes have 0.
- Phase 1 fails on gate A, the function rate, owners, and the M6 to M11 rows (`make phase1-exit`, which also flags the dirty tree). The function rate fell because new fixtures (fixtures/merged-covered-*.xlsx, from T-4148, owned by T-4722) and a pivot probe (t4130-pivot-spill-getpivotdata, owned by T-4154) each put one visibly-off cell under COUNT, COUNTA, COUNTBLANK, ISBLANK, ISTEXT, LEN, ISFORMULA and GETPIVOTDATA. These are new evidence, not regressions.

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,872 (baseline 2,872, lowered by a task PR since the last run);
- oversized files: 8 (= last run).

Merged since ebe81baf: 134 commits, from #3587 to #3861, #3746 among them. None were reverted. All were committed by GitHub with their PR's `(#n)` suffix, and 24 of them are task-only `tasks:` PRs. Among them:
- decisions 0325 (#3727: a probe cell counted for want of a math value is owned by its math task), 0329 (#3786: target Excel's 2026-11 nested-cell reads), 0331 (#3801) and 0332 (#3809): busy lanes per tier, see "Needs owner"; and SPEC-0035 (#3779: the fetched-suite gate);
- M6: T-4569's SPEC-0014 amendment (#3679) and T-4570's XLOOKUP fix (#3757), which closed last run's 108 M6 probe cells;
- pivots (M16): export of caches, records and definitions (T-4176, #3800, the round-trip regression above), rendering, show-values-as, layout and number formats (T-4151, T-4163, T-4168, T-4169, T-4698), and property tests (T-4155);
- M9 kernel fixes: T-3554, T-3577, T-3740, T-3862, T-4242, T-4502, T-4510, T-4537, T-4541, T-4590, T-2788 (PRICE).

Blocked: 45 on `main` with this PR's files (43 last run). New: T-4689 and T-4690, both M15 tasks deferred to Phase 2. No task on `main` is `Blocked because: review-round limit`.

Open PRs: 20 at 06:35 UTC (24 last run), 12 of them drafts.
- Orphaned-PR check (`agent/*` or `claude/*`, not draft, not queued; idle 4 h, or `queue-dropped`):
  - #3847 (T-4714, `queue-dropped`). Its session's landing notes asked for a catch-up. I merged `origin/main` and resolved the `eval/baseline/probes.txt` conflict: I kept main's lines and dropped the five lines it fixes. CI was green on 2539f17f. I ran the security, design and excel-fidelity reviewers myself on that head, and all three returned APPROVE. Then I enqueued it (SQUASH).
  - #3858 (T-4765, `queue-dropped`) was caught up by its own session and merged at 06:14 UTC.
  - The other non-draft agent PRs were younger than 4 h. The oldest is #3822, whose head is from 02:44 UTC.
- Duplicate check: no two open PRs carry the same task id.
- Stale drafts (head 24 h or more, decision 0264):
  - #3443 (T-4408, head 2026-11-05 23:15 UTC, 31 h): bucket 1, superseded by #3806. Closed with a comment; nothing deleted.
  - #3390 (T-4355, head 2026-11-05 17:22 UTC, 37 h): bucket 1, superseded by #3833 (and #3837 closed its salvage task T-4672). Closed with a comment; nothing deleted.
  - #3488 (T-4425, head 2026-11-06 04:59 UTC, 25 h): bucket 5. T-4425 is done on the branch and CI is green, but no reviewer has run (the table is all pending). Its probe and the IMTAN/IMCOT switch are not on `main`, so this PR files T-4795 to land them from a fresh branch.
  - #3428 (T-3117, head 2026-11-05 22:16 UTC, 32 h): bucket 5 (time-boxed). Its probes-numeric case, sweep modes and financial.md finding 7 are not on `main`, so this PR files the salvage task T-4796.
  - #3177 (T-3933, 64 h): bucket 5. Its salvage task T-4387 is still open.
  - #3019 (T-3963, 59 h): bucket 3, owner needed. The T-4239 entry below is on `main`.
- Younger drafts: #3505, #3700, #3739, #3765, #3776, #3823, #3825 and #3859.

Queues:
- `make ready-tasks LANE=functions` printed 16 (≥ 3), so the gap-triager did not run.
- Current-milestone lanes (M6): every failing M6 exit cell is owned. engine, the critical lane, had 1 ready task, so I ran the planner for engine Phase 1 exit work (see the PR). api had 4 ready, harness 19, quality 9 and research 165.
- `TIER=sonnet` printed 220 (≥ 10).

Upstream IronCalc: `scripts/upstream-log.sh` listed no commits after the last triaged one, so there was nothing to file.

Issues: the one open issue, #2277, already has its task (T-0496), so I ran no intake.

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 4 pending cases. `fill -b excel-graph --suite probes` wrote 0 sidecars. Excel for the web again refused t0481-curly-quote-names-c and typed-if-one-argument, and the other two are known unfillable (T-1075).
- As before, ironcalc-calc's 1 pending case is Excel-authored, and poi's are web-vs-desktop conflicts (decision 0019), so I refilled neither. calipers and corpus-bench are scored from their own cached values.

Spot-audit: the last was 2026-11-01, so none is due. The next is on or after 2026-11-08.

**Critical lane:** engine

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M6.
**Owner ruling, 2026-11-04: until Phase 1 exits, the critical lane is `engine` only.** Research probes and harness
Gate A work close no Phase 1 exit criterion, and ranking them above `functions` kept workers off the exit blockers.
The steward keeps this line as it is until the owner changes it.
- First: P0 T-4798 (engine), the round-trip regression on poi/54436.xlsx from #3800.
- Every exit cell left is owned by open work:
  - M6's 34 probe cells: #3847 (T-4714, queued) and T-4739 to T-4742;
  - M7's 28: T-4736, T-4737 and T-4738 (engine, decision 0329);
  - M10's 28: T-4419 (functions-text), T-4460 and T-4466 (research), and T-4333 (blocked: time-boxed);
  - M9's 1,817 visibly-off probe cells: the functions-stat and functions-financial fit work.
- A directive-5 crash P0 (a fuzz or panic report) still comes first. The session that files it starts it at once,
  rather than leaving it to wait behind these by milestone.
- api, harness, quality and research have no M6 exit work on the critical path. perf stays out: it is Phase 2 work.)

## Needs owner

- **Re-apply the worker routines from docs/process/routines.md** (added 2026-11-07, T-3821). Decision 0331 makes a
  lane busy per tier, and both worker prompts now say so ("lanes that are busy for your tier"); decision 0332 (T-3824)
  adds that a current-milestone task passes a lane busy only with later-milestone work. Agents never edit routines,
  so please re-apply the sonnet and opus worker prompts.
- **Owner ruling on RATE's keep-r0 secant model, T-4239** (2026-11-04). T-3963 found why RATE's same-state flat
  secants split (Excel keeps the older iterate when a step leaves the residual unchanged) and built the model on draft
  PR #3019: +78 bit-exact of 2,400 literal RATE cells, 59 against 9 on a blind fresh probe. But it moves five
  error-against-number points out of the Gate A envelope, which decision 0265 does not cover, and regresses 37 gated
  cells. Please choose: adopt it with those points accepted, or keep the merged rule (T-4239 has the details).
- **Owner ruling on CUMPRINC's short-range cut, T-4400** (added 2026-11-06). T-4349 (decision 0296) found no rule that decides between the cut `GEOMETRIC_MIN_LEAD` and the geometric sum. Please choose: (1) keep the cut, so decision 0295 stands as a deviation from decision 0188 R4.3 until T-4390 lands; or (2) remove it, which regresses gated cells under decision 0083 (T-4400 lists them).

Pruned 2026-11-04 at the owner's request: the direct pushes to `main` (none since 2026-10-19), the missed steward runs (both
daily firings arriving again since 2026-11-03), the steward prompt drift (matches routines.md since 2026-11-02) and
the RATE flat-secant ruling (T-3916, decided by decision 0265). Ruled 2026-11-06 (decision 0300, keep Excel): T-3794, T-3775,
T-3484 and T-4196, plus T-4277, T-4317, T-4318, T-4392 and T-4285. Ruled 2026-11-06 (decision 0301, keep Excel): T-3544. Resolved 2026-11-06: #3379 (T-3634), which #3701 replaced. The direct-push item's recommendation, a ruleset on
`main` requiring a pull request, remains the owner's to take up or not.

excel-desktop needs are not owner items (decision 0039): T-1016, T-1051 and the other desktop tasks wait in M14 (Phase 4), tracked by T-1074 and T-1075.

## Log
- 2026-10-02 — Owner set `main` as the GitHub default branch and enabled the routines (repo attached).
- 2026-10-01 — `main` created. Routines must be created in the claude.ai UI with the repo attached (docs/process/routines.md).
- 2026-10-01 — M0 bootstrap: fork, eval harness, oracle client, generator P0, CI, agent process.
- 2026-10-02 — T-0200: shared range-number iterator; duplicated lines 9153→8829; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0002: pinned external suite fetcher (`rustsheet-eval fetch`); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0201: criteria/wildcard matching consolidated in functions/criteria.rs; fixed COUNTIF "<>text" on non-text cells; eval 0 regressions, 0 baseline fixes (+2 new spec cases pass).
- 2026-10-02 — T-0003: suite `poi` (Apache POI, fetched, pinned); 334 cases, 3,526 cells, 82.25%; baseline +633 known failures (new suite, 0 regressions); filed T-0111 (timeouts), T-0112 (imports).
- 2026-10-02 — T-0204: coercing range walk shared by VARA/STDEVA family and GCD/LCM; duplicated lines 8829→8610; eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0007: pending cases now run crash-only (panic/timeout/error reported, per-suite crashes column); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0205: D-function criteria routed through shared criteria grammar (CriteriaFlavor::Database); eval 0 regressions, 0 fixed (+1 new smoke case passes).
- 2026-10-02 — T-0005: transitive volatile skipping + METADATA-pinned clock (NOW/TODAY now scored); NOW() keeps sub-second precision; ironcalc-calc +2 scored cells pass; eval 0 regressions, baseline unchanged.
- 2026-10-02 — T-0008: `rustsheet-gen --policy numeric-rules` → suite `probes-numeric` (2,738 hypothesis-tagged probe cells, pending oracle); eval 0 regressions, 0 fixed.
- 2026-10-02 — T-0202: catalog extended with 18 M2 functions + pair-repeat groups in generator; eval 0 regressions, 0 fixed (tools-only).
- 2026-10-03 — T-0902: eval snapshot reader decodes _xHHHH_ escapes (CHAR 29 cells fixed; ironcalc-calc known failures 1909 -> 1880)
- 2026-10-03 — T-0100: SPEC-0001 public API v0 (facade crate) accepted; filed T-0113; docs only, eval unchanged.
- 2026-10-03 — T-0900: BESSELJ/BESSELY bit-exact for |x|<8 via NR approximations (ironcalc-calc known failures 1880 -> 1761); |x|>=8 filed as T-0908
- 2026-10-03 — T-0113: new `rustsheet` facade crate with typed ErrorKind/XlError/RefError and single ERROR.TYPE table; eval 0 regressions, 0 fixed.
- 2026-10-03 — steward: health snapshot; filed T-0114..T-0117 (api/harness ready queues), T-0910/T-0911 (IronCalc ports); UPSTREAM.md triaged through 9b1ae3fd. Open PRs #11, #13 are blocked docs-only drafts from other sessions (left untouched).
- 2026-10-03 — T-0910 (partial): TIMEVALUE of "date time" text returns serial fraction as Excel does (ironcalc-calc known failures 1761 -> 1760); fractional seconds still need oracle evidence
- 2026-10-03 — T-0114: API v0 `refs.rs` (SheetId/CellRef/RangeRef/Target, pure A1 parse/print); eval 0 regressions, 0 fixed.
- 2026-10-03 — T-0912: MOD uses exact remainder (fmod) as Excel; ironcalc-calc known failures 1760 -> 1756 (fixed 4, 0 regressions)
- 2026-10-03 — T-0115: API v0 typed Value/Array and engine cell conversion; eval 0 regressions, 0 fixed.
- 2026-10-03 — T-0914 (partial): SINH/COSH/TANH use Excel's exp formulation (ironcalc-calc known failures 1756 -> 1731, fixed 25, 0 regressions); T-0916 (IRR/RATE/XIRR) found not fittable without the oracle; filed T-0917 for the rest
- 2026-10-03 — T-0111: VLOOKUP search column bounded by last used row; 2 POI timeout cases re-included (poi known failures 633 -> 655 from 24 newly scored cells, 0 regressions); TANH 0.9<=|x|<1 uses e^2x form (fixes main's poi G1360 regression); baseline also records ~26 crash-only POI cases (filed T-0118, T-0119)
- 2026-10-03 — T-0911 (ROW/COLUMN part): ROW/COLUMN of multi-cell references return vectors (ported ironcalc b6d34abc); ironcalc-calc 21 newly checked cells, 0 failures, 0 regressions; rest filed as T-0918
- 2026-10-03 — T-0919: @ROW/@COLUMN skip materialising whole-column vectors (165 ms -> 0.17 ms); eval delta none
- 2026-10-03 — T-0918: ported ironcalc INDIRECT (R1C1 + INDIRECT.xlsx) and text-on-arrays fixes; ironcalc-calc +311 newly checked cells with 0 failures, 0 regressions; remaining ports filed as T-0920
- 2026-10-04 — T-0920 (partial): ported POWER-accepts-ranges (daf5f6c6) and implicit-intersection export (9212095d); ironcalc-calc +51 newly checked cells (47 pass, 4 NumericUlp baselined), 0 regressions; finite-number parse and --A1:B4 remain
- 2026-10-04 — T-0118: importer no longer panics on POI workbooks (typed errors, optional style sections); poi known failures 655 -> 647, 0 regressions beyond 4 crash-kind relabels (decision 0012); duplicated lines 8610->8420.
- 2026-10-04 — T-0913: eval snapshot reader decodes rich-value `vm` errors (#CALC!/#SPILL! cached as #VALUE!); ironcalc-calc known failures 1735 -> 1727 (8 fixed), 0 regressions; engine unchanged
- 2026-10-04 — T-0101: removed lifetime from Model/UserModel/Parser/Lexer and the Python binding's leak helper; eval delta none (0 fixed, 0 regressions)
- 2026-10-04 — steward: health snapshot; queues healthy, no new upstream commits, no oracle credentials.
- 2026-10-04 — T-0923: TIME divides before dropping whole days (+7 cells, ironcalc-calc 97.60%); triaged T-0922..T-0929 (no oracle needed)
- 2026-10-04 — T-0102: API v0 `Workbook` read path + xlsx load/save with zip-bomb limits; eval adapter on the facade; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0924: shared accurate erf/erfc + normal CDF/PDF kernel (ERF*, NORM.DIST/S.DIST, LOGNORM.DIST, GAUSS, Z.TEST); ironcalc-calc known failures 1720 -> 1632 (88 fixed), 0 regressions
- 2026-10-04 — T-0103: API v0 `Workbook::edit`/`set` with undo-log rollback and diff-based `ChangeSet`; eval adapter builds specs via `edit`; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0104: Python bindings on API v0 (`rs.Workbook`, `edit()` transactions, `XlError`, typed exceptions, GIL released); eval unaffected; filed T-0931 (parser nesting stack overflow).
- 2026-10-04 — T-0928: EXPON.DIST cumulative uses -expm1; ironcalc-calc known failures 1632 -> 1626 (6 fixed), 0 regressions
- 2026-10-04 — T-0930: API edit follow-ups (diff split into edit/diff.rs, sheet-position map, error classifier next to RefError, engine take/restore tests, evaluation counter, bench moved); eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0926: GAMMALN/GAMMALN.PRECISE via libm lgamma; ironcalc-calc known failures 1626 -> 1578 (48 fixed), 0 regressions; 12 cells need oracle
- 2026-10-04 — T-0004 (partial): excel-graph oracle live (app-only, `Sites.Selected` site); fixed site drive addressing; T-0004, T-0106..T-0110, T-0203 unblocked.
- 2026-10-04 — T-0932: eval wall time 72.5 s → 71.1 s across T-0930; edit bench median baseline (100k formulas): recalc 13.4 ms, tracked set 16.5 ms, diff overhead 3.0 ms; KeyHasher deduplicated; eval 0 regressions, 0 fixed.
- 2026-10-04 — T-0009: excel-graph sidecars for probes (3) and probes-numeric (4); probes 71.11% (13 new-coverage known failures), probes-numeric 60.19% (not gated); 0 regressions; excel-semantics: 9 entries VERIFIED (snapping corrected), 4 new; inputs with ≥1E+308 literals fixed (Excel rejects them). T-0106/7/8/10 now ready.
- 2026-10-04 — Owner updated the routine prompts to the text in `docs/process/routines.md` (verified identical). Decision 0014: Excel is always right; every quirk found is recorded in docs/excel-semantics/.
- 2026-10-04 — T-0013 (+ T-0010): every oracle workbook carries an embedded canary (poisoned cached values) and is refused unless Excel recalculated it; canary exposed that Excel-authored workbooks were never saved by the Graph session (old flow returned cached values) — fixed with a post-calculate no-trace edit; Graph calls retry dropped connections (T-0010); eval delta none
- 2026-10-04 — T-0004 (partial): smoke oracle-filled (canary-verified); Excel disproved D-function criteria trimming (engine fixed, criteria.md entry); eval now lets oracle values replace hand values keyed without sheet quotes; smoke 38/38 vs Excel, 0 regressions
- 2026-10-04 — Owner review: filed T-0016..T-0023, T-0402 (model tiers, board conflicts, task ids, milestone-order triage, POI oracle fill, evidence coverage, fuzzing, reviewer models, perf gate); T-0203 and T-0004 raised to P0; critical lane engine, harness, research, perf (API v0 done, api lane empty).
- 2026-10-04 — T-0022: security/architecture/quality reviewers on Sonnet, fidelity/performance pinned to Opus (decision 0015); eval delta none.
- 2026-10-04 — T-0017: `make new-task` assigns collision-free ids (all origin/* branches counted; decision 0016); eval delta none.
- 2026-10-04 — T-0016: docs/tasks/BOARD.md no longer committed (git-ignored, `make board` renders locally; decision 0017); eval delta none.
- 2026-10-04 — T-0023: tasks carry `tier: sonnet | opus`; sonnet (hourly) and opus (2-hourly) workers replace the lane workers; stuck sonnet runs hand off (decision 0018). Needs owner: re-apply docs/process/routines.md. Eval delta none.
- 2026-10-04 — T-0004 done: oracle calibration over ironcalc-calc (71,765 cells, 99.96% bit-exact; 7 day-first-locale cached cells, 4 web-vs-desktop INDIRECT, 18 last-digit float); canary pins en-US; probes/date-text-locale added (probes known failures 13 -> 18, new coverage); filed T-0024
- 2026-10-04 — T-0108: Excel 1900 leap-year serials in DATE/DAY/MONTH/YEAR/formatting; probes known failures 18 -> 15 (A1-A3 fixed), ironcalc-calc 1578 -> 1578, 0 regressions; filed T-0125
- 2026-10-04 — T-0108 follow-up: added probes/dates-early-1900 (oracle-filled, 16 cells, 4 known failures -> T-0125); fixed EOMONTH early serials and DATE(1900,1,0)
- 2026-10-04 — T-0019: excel-graph sidecars for 40 POI workbooks (`oracle_dir`, committed in eval/suites/poi/oracle); poi pending 237 -> 198, known failures 647 -> 959 (new coverage, 0 regressions; 257 web-vs-desktop conflicts left pending, decision 0019); harness now parses JSON floats exactly (serde_json `float_roundtrip`; oracle values had read 1 ulp off, probes-numeric 60.19% -> 68.08%); quirk: SIN/COS at multiples of 90°.
- 2026-10-04 — T-0206: probes/criteria-operands oracle-filled (25 cells); fixed <,>,<=,>= with boolean/error operands and D-function explicit '=' exactness; 0 regressions, probes 25/25 new cells pass
- 2026-10-05 — T-0028: agents check PR mergeability after every push, event and merge to main, and resolve conflicts at once (decision 0020); `make check` runs typos; eval delta none.
- 2026-10-05 — T-0015: spec2xlsx output deterministic (pinned core.xml creation time); eval delta none.
- 2026-10-05 — T-0027: shared-formula members shown shifted in eval show/snapshot (engine `shift_formula_text`); eval delta none.
- 2026-10-05 — T-0203: gen-m2 oracle-filled and gated (30 cases, 12,000 cells, 85.62% bit-exact, 1,726 known failures, new coverage); IF arity fixed in the catalog; T-0208 fixed (oracle `#SPILL!` read by key name; 74 cells); 14 gap tasks T-0209..T-0222; unknown rich errorTypes read as `#UNDECODED!` and stay unscored (poi 951 -> 950, T-0030).
- 2026-10-05 — T-0123: names may contain `\` (en-US lexer, identifier validity); poi known failures 950 -> 935 (15 fixed, 0 regressions).
- 2026-10-05 — T-0031: salvaged stale draft PRs #11/#13/#38 (T-0905, T-0907 restored; financial.md; T-0901/T-0925 findings) and closed them; eval delta none.
- 2026-10-05 — Log frozen here (decision 0024, T-0033); later work: `git log --first-parent main` and the steward's daily summary.
- 2026-10-05 — Owner re-applied the T-0023 routines (sonnet hourly, opus every 2 h); T-0032: sonnet queue buffer (≥10 ready sonnet tasks; `make ready-tasks`, decision 0021).
