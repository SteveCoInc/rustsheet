# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-11-07, 19:42 UTC run (M6 is current)

This is the second run of the day. It follows #3872, the 05:53 UTC run, squash-merged as 351f8f78. It covers the 134 commits merged to `main` after 42d9bdaa (#3861, where that run's list stopped) up to c709e9d2 (#4006), the head when this run started. `make eval` ran on 04f97f7d with a clean tree. That commit records #3872's head as a parent and has c709e9d2's tree.

- All 21 suites were scored. `make eval` now fetches missing gate suites itself (T-4726, decision 0344), so calipers, corpus-bench, poi and both pivot corpora were fetched on the first run. Timeouts: 0 in every suite.
- Eval gate: `make eval-lower-visible-lock` shows 0 regressions and 0 fixed in every gated suite. It lowered the visible-parity lock (decision 0093) from 8,275 allowed in 326 cases to 6,160 in-scope visibly-off cells in 286 cases (0 in new cases; re-run after merging `main` at review). The allowance had grown since the morning run through the new cases that task PRs recorded.
- Round trip: `rustsheet-eval roundtrip --check` reports no regression. poi/54436 (this morning's P0) is still owned by T-4825 (P0, todo; #3961 made it the one owner). T-4961 (#3988) owns the ratchet that let #3892 record the regression.
- **Dashboard not regenerated.** `make progress-report` exits 1: "tasks.json by_milestone counts milestone M17, which docs/milestones.md has no heading for". #3878 (tasks: plan M17) filed T-4808 to T-4822 under M17 before decision T-4808 adds M17 to docs/milestones.md. This is a refusal, not a stale run (SPEC-0022), so docs/progress.md keeps the morning run's numbers. I did not add an M17 heading myself, because its scope and phase are T-4808's owner ruling (see "Needs owner"). This PR files T-5018 so that `tasks.py validate` catches this class of error in CI.

CI on `main` is green: of the last 40 `ci.yml` push runs on `main`, 39 succeeded and the newest was still running. The latest scheduled Nightly eval (2026-11-07 13:01 UTC, f4ef38d0) and Nightly fuzz (2026-11-07 10:39 UTC, dd67308d) runs both succeeded.

Milestones: the current milestone stays M6 (decision 0027 order), and its exit fails only on its probes.
- M6's probes have 82 visibly-off cells. 79 are the GETPIVOTDATA probes t2085-getpivotdata-layouts, -items and -data-fields (from T-2085), owned by T-4804 (#3950, orphaned and conflicting), T-4805 and T-4806. The other 3 are t4742-countif-array-range-arg, owned by T-4826, T-4863 and T-4946. The morning run's 34 cells are closed: #3847 (T-4714) and T-4739 to T-4742 merged.
- `make exit-owners MILESTONE=M6` reports 0, as do M10 and M11.
  - M7 listed 33 cells: 26 gen-m7/21b4eef18424ce94 spill cells and 7 probe cells, which T-4737 superseded under decision 0329 item 3. This PR names them in T-4913 (the owner-tool fix for decision-superseded cells), and M7 now reports 0.
  - M8 still lists t4618-order-after-active-backward Q!A1. T-3610, an M9 harness task, owns it and names it in its acceptance criteria.
  - M9 listed 2. The COUNTBLANK fixture cell is still owned by T-4722 (M11). New is t3409-r4-gamma-inv-search-sweep Calc!A234, a CHISQ.INV.RT point where ours equals the math sidecar and Excel is about 2.7E-4 off. It is inside decision 0337's R3 region (shape 1E9 or more) and lacks only its math-over-excel.tsv mark, as do Calc!A226 and its neighbours. This PR files T-5015 to mark them.
- The frozen M1 list has 0 known failures (89/89 cases), and the M0–M2 probes have 0.

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,872 (baseline 2,872, = last run);
- oversized files: 8 (= last run).

Merged since 42d9bdaa: 134 commits, from #3488 to #4006, #3872 among them. None were reverted, and 17 of them are task-only `tasks:` PRs. Among them:
- decisions 0333 (#3867: a case that fails whole is owned by any block naming it), 0336 (round-trip delta violations ratchet in the baseline), 0337 (#3915: GAMMA.INV's huge-shape closed-form branch follows the math, R3), 0342 (excel-saved oracle downloads), 0344 (#3999: CI's eval job gates poi and corpus-bench from a cached fetch) and 0346 (the Nightly eval fetches, runs and checks the corpora);
- M6: T-4714 (#3847) and T-4739 to T-4742, which closed the morning run's 34 probe cells;
- M7: T-4737 and T-4738 (nested GROUPBY/PIVOTBY reads, decision 0329);
- pivots (M16): slicer and timeline caches as filters (T-4188), PivotChart caches (T-4187), the engine-exported round-trip suite (T-4177), GETPIVOTDATA error arguments (T-4823), and the plan for M17 workbook locales (#3878).

Blocked: 49 on `main` with this PR's files (50 at #3872's merge). T-4909 merged done. No task on `main` is `Blocked because: review-round limit`.

Open PRs: 29 at 20:35 UTC (20 this morning), 17 of them drafts.
- Orphaned-PR check (`agent/*` or `claude/*`, not draft, not queued; idle 4 h, or `queue-dropped`; no PR carried `queue-dropped`):
  - #3979 (T-4801, idle 4.9 h, clean, green). I ran the panel myself on 111e336eb. Security and design returned APPROVE. Excel fidelity returned REQUEST_CHANGES: lookup.md claimed that a lifted `number`/`order` zips with the grid, which the probes cannot show. I pushed cb813050, which marks that claim UNVERIFIED and adds the shapes that tell the cases apart to T-4951. All three reviewers then returned APPROVE on cb813050. Another session enqueued it once CI was green, and it merged as d2088ed3.
  - #3958 (T-4143) was a duplicate: T-4143 merged through #3957. Its CI was red and its branch idle 6.8 h, so I closed it with a comment and deleted nothing.
  - #3963 (T-4901), #3953 (T-1871), #3952 (T-4807), #3950 (T-4804) and #3940 (T-4857) conflict with `main`. #3952 is also red, and #3953 and #3940 have no CI on their heads. I commented each blocker. For #3963 I also gave the exact resolution of its numbers.md paragraph; its baselines still need re-recording.
  - #3911 (T-3351) conflicts, and its task is `status: blocked` by the reviewers' request, so it is not adoptable.
  - #3937 (T-4856 and T-4257), #3896 (T-4635) and #4001 (T-4965) were younger than 4 h.
- Duplicate check: apart from #3958, no two open PRs carry the same task id.
- Stale drafts (head 24 h or more, decision 0264):
  - #3505 (T-3900, head 2026-11-06 06:21 UTC, 38 h): bucket 4, a task-only PR that merges cleanly with green CI. I marked it ready for review. Its task is `blocked`, so the orphan rule will not enqueue it.
  - #3700 (T-4646, head 2026-11-06 17:48 UTC, 27 h): bucket 5. Its design verdict is pending and it holds no evidence to salvage. I commented once.
  - #3428 (T-3117, salvage task T-4796), #3177 (T-3933, salvage task T-4387) and #3019 (T-3963, bucket 3, the T-4239 entry below): as the morning run left them.
- Younger drafts: #3739, #3765, #3776, #3825, #3859, #3907, #3924, #3929, #3936, #3939, #3966, #3973 and #4016.

Queues:
- `make ready-tasks LANE=functions` printed 76 (≥ 3), so the gap-triager did not run.
- Current-milestone lanes: engine, the critical lane, printed 0 ready, so I ran the planner. It filed nothing: no engine task is open in M6 to M10, and every M6 exit cell has an owner in another lane. The engine count is 0 because T-1089, T-3636 and T-4144 (M11) hold the three opus slots, and decision 0332 lets only current-milestone work pass them. Among sonnet tasks, T-4935 is claimed and T-0490 waits on T-0489. api had 3 ready, harness 17, quality 8 and research 201.
- `TIER=sonnet` printed 233 (≥ 10).

Upstream IronCalc: `scripts/upstream-log.sh` listed no commits after the last triaged one, so there was nothing to file.

Issues: the one open issue, #2277, already has its task (T-0496), so I ran no intake.

Oracle: excel-graph credentials are present.
- probes: 4 pending cases. `fill -b excel-graph --suite probes` wrote 0 sidecars. Excel for the web again refused t0481-curly-quote-names-c and typed-if-one-argument (FileCorruptTryRepair), and the other two are known unfillable (T-1075).
- As before, I refilled neither ironcalc-calc's 1 pending case (Excel-authored) nor poi's (web-vs-desktop conflicts, decision 0019).

Spot-audit: the last was 2026-11-01, so none is due. The next is on or after 2026-11-08.

**Critical lane:** engine

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M6.
**Owner ruling, 2026-11-04: until Phase 1 exits, the critical lane is `engine` only.** Research probes and harness
Gate A work close no Phase 1 exit criterion, and ranking them above `functions` kept workers off the exit blockers.
The steward keeps this line as it is until the owner changes it.
- First: P0 T-4825 (engine, M16), the round-trip regression on poi/54436.xlsx from #3800 (T-4798 closed as its duplicate).
- Every exit cell left is owned by open work:
  - M6's 82 probe cells: T-4804 (#3950), T-4805 and T-4806 (GETPIVOTDATA, 79), and T-4826, T-4863 and T-4946 (3);
  - M7's 20 visibly-off probe cells, plus the 26 gen-m7 spill cells that exit-owners also lists: T-4736 (draft #3859)
    and T-4913 (decision-superseded cells, decision 0329);
  - M10's 28: T-4419 (functions-text), T-4460 and T-4466 (research), and T-4333 (blocked: time-boxed);
  - M9's 2,123 visibly-off probe cells: the functions-stat and functions-financial fit work, with T-5015 to mark the
    unmarked R3 points of the t3409 sweeps.
- A directive-5 crash P0 (a fuzz or panic report) still comes first. The session that files it starts it at once,
  rather than leaving it to wait behind these by milestone.
- api, harness, quality and research have no M6 exit work on the critical path. perf stays out: it is Phase 2 work.)

## Needs owner

- **Decision T-4808: add M17 (workbook locales beyond en-US) to docs/milestones.md** (added 2026-11-07 evening). #3878
  filed T-4808 to T-4822 under milestone M17, and docs/milestones.md has no M17 heading yet. Until it does,
  `make progress-report` refuses the run, so docs/progress.md cannot be regenerated (this run kept the morning's
  numbers). T-4808 asks the owner to rule on M17's locales, phase and interim API status. A ruling, or an interim
  M17 heading the owner approves, unblocks the dashboard. T-5018 stops the next such PR in CI.
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
