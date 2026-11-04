# Status

The steward routine owns the snapshot and the critical-lane line and rewrites them daily; agents may
still add or remove "Needs owner" entries. PRs no longer append log lines (decision 0024): the record
of merged work is `git log --first-parent main` plus each task's Outcome, and the steward lists what
merged since its last run.

**Progress dashboard:** [docs/progress.md](progress.md), regenerated daily by the steward.

## Snapshot — 2026-11-04, 05:50 UTC run (M2 is current)

This run follows #3042, the 2026-11-03 19:50 UTC run (squash-merged as e57b6fa5 at 20:50 UTC). It covers the 56 commits merged to `main` after e57b6fa5 up to a7b115e9 (#3100), the head when this run started; `make eval` ran on a7b115e9 (clean tree).

`make eval` ran on a7b115e9 (clean tree).
- All 17 suites were scored. calipers, corpus-bench and poi were again not fetched in this container, so the first `make eval` skipped them; I ran `rustsheet-eval fetch` for the three and ran `make eval` again. Timeouts: 0 in every suite.
- Eval gate: `make eval-lower-visible-lock` shows 0 regressions and 0 fixed in every gated suite. It lowered the visible-parity lock (decision 0093) from 2,435 allowed in 252 cases to 2,325 in-scope visibly-off cells in 247 cases (0 in new cases).
- The dashboard has this run's numbers (`make progress-report`; the judgment blocks pass `--check`).

CI on `main` is green: the last 45 `ci.yml` push runs on `main` succeeded, through a7b115e9.

M2 is still current. M0–M2 probes (decision 0051) have 154 known failures (156 last run), all 154 visibly off, in four cases:
- t3584-match-fold-weightless (125, 133 last run), owned by T-3904; T-4013 (#3084) split the MATCH classes out of it and landed.
- t3465-collation-holdout (22), owned by T-3904, T-3662, T-3305 and T-3381.
- t3929-wildcard-fold-criteria (6), new in the row with T-3929 (#3051), owned by T-3904.
- t3895-collation-script-digits (1), owned by T-3904.
- `make exit-owners MILESTONE=M2` reports 0. 0 M0–M2 cases did not run. The frozen M1 list has 0 known failures (89/89 cases).

Hygiene:
- lint-debt: 91 allow entries (= last run);
- duplicated lines: 2,880 (baseline 2,880; = last run);
- oversized files: 8 (= last run).

Merged since #3042 (e57b6fa5): 56 commits up to a7b115e9, from #2877 to #3100. None were reverted. All were committed by GitHub, each with its PR's `(#n)` suffix; 13 of them are task-only `tasks:` PRs. Among them: T-4013's MATCH classes for `=`, COUNTIF and XMATCH (#3084), T-3929's D-function criteria fold (#3051), T-3678's Gate A s1 tables for TIME and friends (#3047), T-3686's Gate A tables for MDETERM, MINVERSE and MMULT (#3083), and T-3411's IRR fallback (#3063).

Blocked: 47 on `main` (47 last run). No task on `main` is `Blocked because: review-round limit`.

Open PRs: 15 at 05:56 UTC (21 last run), 11 of them drafts.
- Orphaned-PR check (`agent/*` or `claude/*`, not draft, not queued, idle 4 h): none at 06:20 UTC. #3079 (T-3983, idle since 02:57 UTC) conflicts with `main` on `eval/baseline/probes.txt` only and has no CI run; it becomes an orphan at 06:57 UTC. #3082 (T-3990, idle since 03:10 UTC) is green; it becomes an orphan at 07:10 UTC. #3101 and #3102 opened within the hour. No PR carries `queue-dropped`.
- Drafts older than 24 h: #1885, #2529, #2573, #2624, #2635, #2640, #2671, #2717 and #2788 stay drafts.
- Duplicate check: no two open PRs carry the same task id.

Queues:
- `make ready-tasks LANE=functions` printed 0 ready, because every functions-* lane has an agent branch that moved in the last 2 h; 305 functions-* tasks are todo on `main`, and opus gap triages landed four times since the last run (#3064, #3077, #3080, #3095). So I did not run the gap-triager (judgment: new tasks would only queue behind the claims).
- engine, research and harness printed 0 for the same reason; 45 engine, 170 research and 54 harness tasks are todo on `main`. So I did not run the planner.
- api printed 9, perf 17, quality 13, and `TIER=sonnet` 39 (≥ 10).

Upstream IronCalc: `scripts/upstream-log.sh` listed no commit after the last triaged one, so no port task.

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
- engine: the M2 exit cells (T-4040, T-4024; T-3904 is done), M5 (T-3891, T-3997, T-3950), M7 (T-3418) and M8
  (T-4020, T-1683, T-3856), all P0. T-4039 (M2) and T-3932 (M5) keep their priority because a live branch works
  them already.
- functions: the M10 numeral-digit and locale cells (T-3710, T-3711, T-3089, T-2959 at P0, T-3598 done; T-3666 at P1,
  claimed), M6's TIME wrap (T-3983, P1, claimed, PR #3079) and M9 (worked in its own session).
- A directive-5 crash P0 (a fuzz or panic report) still comes first: the session that files it starts it at once,
  rather than leaving it to wait behind these by milestone.
- research: only the probes that own exit cells, now P0 (T-3902, T-3739); T-3381 and T-3305 stay with the M2
  collation work.)
- engine: the 154 visibly-off M0–M2 cells: T-3904 (weightless characters in `=`, COUNTIF and XMATCH).
- research: the M2 probe queue (T-3662, T-3305, T-3381).
- harness: Gate A (SPEC-0029).
- api and quality have no M2 work on the critical path. perf stays out: Phase 2.)

## Needs owner

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
