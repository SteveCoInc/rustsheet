# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-11-05, 05:58 UTC run (M2 is current)

This run follows #3224, the 2026-11-04 19:43 UTC run (squash-merged as 54ca14c0 at 21:05 UTC). It covers the 59 commits merged to `main` after 54ca14c0 up to c3de9423 (#3289); `make eval` ran on 70d01bb1 (#3288), the head when this run started, on a clean tree.

- All 17 suites were scored. calipers, corpus-bench and poi were again not fetched in this container, so the first `make eval` skipped them; I ran `rustsheet-eval fetch` for the three and ran `make eval` again. Timeouts: 0 in every suite.
- Eval gate: `make eval-lower-visible-lock` shows 0 regressions and 0 fixed in every gated suite. It lowered the visible-parity lock (decision 0093) from 2,456 allowed in 242 cases to 2,042 in-scope visibly-off cells in 230 cases (0 in new cases).
- The dashboard has this run's numbers (`make progress-report`; the judgment blocks pass `--check`).

CI on `main` is green: the last 60 `ci.yml` push runs on `main` succeeded, through 70d01bb1.

M2 is still current. M0–M2 probes (decision 0051) have 14 known failures (2 last run), all visibly off, all in one case added this period:
- probes/t4224-doubled-xlpm-prefix (T-4224's probe, #3283), owned by T-4288 (claimed, draft #3292). The t3465 collation hold-out cells of last run now pass.
- `make exit-owners MILESTONE=M2` reports 0, and so does `MILESTONE=M6`. 0 M0–M2 cases did not run. The frozen M1 list has 0 known failures (89/89 cases).
- M6's probe row is no longer met: the new t4236 pivot date-group probes (#3239) hold 120 visibly-off cells. T-4260 (#3290) landed after this run's eval; T-4261, their other open owner, unclaimed at P3, is raised to P0 here under the owner's 2026-11-04 exit-blockers rule (#3126).

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,878 (baseline 2,880, unchanged). Not lowered here: `quality/` is a protected path (`check_governance.py`), so a task PR records the drop;
- oversized files: 8 (= last run).

Merged since #3224 (54ca14c0): 59 commits up to c3de9423, from #3196 to #3289. None were reverted. All were committed by GitHub, each with its PR's `(#n)` suffix; 12 of them are task-only `tasks:` PRs. Among them: T-4258's lone-mark collation (#3262), T-4246's engine date groups (#3258), T-4235's range-group step rule (#3238), T-4264's OLE compound-file classification (#3279), T-3367's NPER at tiny rates under decision 0188 R3 (#3274), T-4233's comparison text in the work budget (#3244) and T-2898's move of Model and UserModel to `rustsheet.legacy` (#3252).

Blocked: 29 on `main` (25 last run). No task on `main` is `Blocked because: review-round limit`.

Open PRs: 11 at 06:24 UTC (7 last run), 6 of them drafts.
- Orphaned-PR check (`agent/*` or `claude/*`, not draft, not queued, idle 4 h): none idle 4 h (#3278, T-4249, head pushed 03:50 UTC, is the oldest; #3284 and #3293 are younger).
  - #3266 (T-4248) carries `queue-dropped` and is now dirty after #3289. Another opus session adopted it at 06:05 UTC (its `make catch-up` merge aa4adf84, CI green), so I left it to that live session rather than race it.
- Drafts older than 24 h: none. #3105 (T-3141, blocked on T-4059; head 2026-11-04 07:13 UTC) is the oldest and reaches 24 h after this run. #3019 (T-3963) moved at 2026-11-04 19:50 UTC and is now titled parked on the owner's T-4239 ruling ("Needs owner" below).
- Younger drafts: #3294 (T-4290), #3292 (T-4288), #3267 (T-3783), #3211 (T-4213) and #3177 (T-3933, parked by decision 0267).
- Duplicate check: no two open PRs carry the same task id.

Queues:
- `make ready-tasks LANE=functions` printed 0: functions-stat, -financial and -text are busy (114, 30 and 16 unclaimed ready tasks behind their claims), and functions-math, -criteria, -engineering, -lookup, -logic and -date have no ready task. So I ran the gap-triager for those six lanes (GAP_TRIAGE_RESULT).
- engine, api and research printed 0 because their lane slots are taken by branches that moved in the last 2 h; 29 engine, 7 api and 157 research tasks are ready and unclaimed on `main`. So I did not run the planner (judgment: new tasks would only queue behind the claims).
- harness printed 62, perf 18, quality 12, and `TIER=sonnet` 38 (≥ 10).

Upstream IronCalc: `scripts/upstream-log.sh` listed no commits after bec55e1e, so no port task.

Issues: the one open issue, #2277, already has its task (T-0496).

Oracle: excel-graph credentials are present (`rustsheet-oracle status`: excel-graph OK).
- probes: 4 pending cases. `fill -b excel-graph --suite probes` wrote 0 sidecars: Excel for the web refused t0481-curly-quote-names-c and typed-if-one-argument again (`unsupportedWorkbook`), and the other two are known unfillable (T-1075).
- As before, ironcalc-calc's 1 pending case is Excel-authored and poi's are web-vs-desktop conflicts (decision 0019), so I refilled neither. calipers and corpus-bench are scored from their own cached values.

Spot-audit: the last was 2026-11-01, so none is due; next on or after 2026-11-08.

**Critical lane:** engine

(Ordered list read by both tier workers, which then fall back to `functions`. The current milestone is M2.
**Owner ruling, 2026-11-04: until Phase 1 exits, the critical lane is `engine` only.** Research probes and harness
Gate A work close no Phase 1 exit criterion, and ranking them above `functions` kept workers off the exit blockers.
The steward keeps this line as it is until the owner changes it.
- Every owner the 2026-11-04 list raised to P0 is done except T-3739 (research, M8 probe). This run raises the open,
  unclaimed owners of the exit cells that appeared since then to P0, as #3126 did: T-4261 (M6, engine), T-4241 (M7,
  engine), T-4049 (M8, engine), T-4295 and T-4107 (M10, functions-text). T-4288 (M2, engine) keeps P2 because a live
  branch works it (draft #3292); T-4292 (M10) landed in #3291 after this run's eval.
- A directive-5 crash P0 (a fuzz or panic report) still comes first: the session that files it starts it at once,
  rather than leaving it to wait behind these by milestone.
- api, harness, quality and research have no M2 exit work on the critical path; perf stays out: Phase 2.)

## Needs owner

- **Decision 0188 R3 rulings** (no agent, steward included, adds an R3 / `MATH-OVER-EXCEL` case on its own): four tasks
  are blocked on an owner ruling. Each one says where Excel is more than 1e-4 off the math, and which way the task
  would go:
  - T-3794: two-argument ERF where the two values nearly cancel;
  - T-3775: SIN, COS, IMSIN and IMCOS at the 15-digit neighbour of k·pi/2;
  - T-3484: XIRR near a zero root;
  - T-4196: BESSELI at a large x and order 2 or more, and BESSELJ near a root (filed by T-4083, #3157).
  Added 2026-11-04 (19:43 UTC run): these tasks were blocked on the owner but not listed here.
- **Owner ruling on RATE's keep-r0 secant model, T-4239** (2026-11-04). T-3963 found why RATE's same-state flat
  secants split (Excel keeps the older iterate when a step leaves the residual unchanged) and built the model on draft
  PR #3019: +78 bit-exact of 2,400 literal RATE cells, 59 against 9 on a blind fresh probe. But it moves five
  error-against-number points out of the Gate A envelope, which decision 0265 does not cover, and regresses 37 gated
  cells. Please choose: adopt it with those points accepted, or keep the merged rule (T-4239 has the details).

Pruned 2026-11-04 at the owner's request: the direct pushes to `main` (none since 2026-10-19), the missed steward runs (both
daily firings arriving again since 2026-11-03), the steward prompt drift (matches routines.md since 2026-11-02) and
the RATE flat-secant ruling (T-3916, decided by decision 0265). The direct-push item's recommendation, a ruleset on
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
