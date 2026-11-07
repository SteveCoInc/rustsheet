---
id: T-4870
title: "An empty argument of EDATE, EOMONTH, WORKDAY, NETWORKDAYS, WEEKNUM and YEARFRAC follows Excel (probe the Analysis ToolPak #N/A rule)"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A written-empty argument (`EDATE(,1)`, `WORKDAY(1,5,)`) of the Analysis ToolPak date functions EDATE, EOMONTH,
WORKDAY, NETWORKDAYS, WEEKNUM and YEARFRAC gives what Excel gives, decided by a small probe whose predictions are
committed before the fill, so the engine no longer reads an empty argument as 0 where Excel says `#N/A`.

## Context
- Blind spot: the generator only ever drops trailing optionals (`call_args` in
  tools/src/rustsheet_tools/gen/generator.py, "Excel allows omitting trailing optionals only"); it never writes an
  empty argument (`,,` or a trailing `,`). No eval cell calls these six functions with one, so every engine answer
  below is unverified (gap triage 2026-11-07, `make eval` on origin/main 3d991bd3d).
- Excel evidence for the rule to test: every other Analysis ToolPak function probed so far gives `#N/A` for a written-empty
  argument. `MROUND(,1)`, `MROUND(5,)` (arrays.md "MROUND keeps its scalar rules", probes/ceiling-floor-mround-array-lift);
  `RANDBETWEEN(,5)`, `RANDBETWEEN(5,)` (numbers.md, probes/volatile-invariant Calc!I5:I6); `IMSUB("3+4j",)`, `IMDIV(,"3+4j")` (engineering.md,
  probes/eng-imsub-imdiv-blank-j A3:B4); the first argument of GCD, LCM, MULTINOMIAL, IMSUM, IMPRODUCT (`GCD(,4)`,
  coercion.md, probes/gcd-lcm-multinomial-negative-precedence E40:E61), while a later empty argument of those variadic
  ones is skipped. EDATE, EOMONTH, WORKDAY, NETWORKDAYS, WEEKNUM and YEARFRAC are the same ToolPak family (dates.md
  "The Analysis ToolPak-era functions do not take a range").
- Ours today (scratch eval, unverified): `EDATE(,1)` 31, `EDATE(1,)` 1, `EDATE(,)` 0, `EOMONTH(,1)` 59,
  `EOMONTH(40000,)` 40025, `WORKDAY(,1)` 2, `WORKDAY(1,)` 1, `NETWORKDAYS(,1)` 0, `WEEKNUM(,1)` 0, `YEARFRAC(,1)`
  0.00277777777777778, `YEARFRAC(1,)` 0.00277777777777778: every empty read as 0. For the optional ones:
  `WORKDAY(1,5,)` and `NETWORKDAYS(1,10,)` `#VALUE!` (holidays), `WEEKNUM(1,)` `#NUM!` (return_type read as 0),
  `YEARFRAC(13816,16816,)` 8.21111111111111 (basis 0), `WORKDAY.INTL(1,5,1,)` and `NETWORKDAYS.INTL(1,10,1,)`
  `#VALUE!` (holidays). Note the engine reads an empty `weekend` of the .INTL pair as the default weekend but an empty
  `holidays` as `#VALUE!`: one of the two is likely wrong.
- Code: `fn_edate`/`fn_eomonth` (functions/date_and_time/month_shift.rs), `fn_workday`/`fn_workday_intl`
  (date_and_time/workday.rs), `fn_networkdays`/`fn_networkdays_intl` (date_and_time/networkdays.rs), `fn_weeknum`
  (date_and_time/weeknum.rs), `fn_yearfrac` (date_and_time/yearfrac.rs). The `#N/A` check for a written-empty node
  already exists inline in `complex_pair_of` (functions/engineering/complex/mod.rs: `Node::EmptyArgKind` or
  `is_omitted_variable`, so `LAMBDA(a,b,IMSUB(a,b))("1",)` behaves alike); T-4872 and T-4873 need the same check.
  Search first (`rg 'EmptyArgKind' crates/rustsheet-engine/src/functions`) and reuse or extract one helper, never a copy
  (directive 3).
- How it avoids fitting: the outcome of each cell is an error kind or the function's documented default; the probe
  decides which, there is no constant to tune.

## Acceptance criteria
- [x] A probes case (excel-probe skill, excel-graph) covers, for each of EDATE, EOMONTH, WORKDAY, NETWORKDAYS,
      WEEKNUM, YEARFRAC, WORKDAY.INTL and NETWORKDAYS.INTL: each required argument written empty (one at a time, and all
      empty), each optional argument written empty (holidays, return_type, basis), a `LAMBDA(a,b,EDATE(a,b))(1,)` and a
      `LET(e,,EDATE(1,e))` row for the omitted-variable path, and WEEKDAY(45000,) and DAYS(10,) as non-ToolPak
      controls. The empty `weekend` argument stays with T-2591 and is not probed here.
- [x] The engine's predictions for every probe cell (the rule "a written-empty argument of a ToolPak function is
      `#N/A`" for required arguments, today's answers for the rest) are committed in a commit before the oracle fill.
- [x] After the fill, every cell of the probe passes; dates.md gets one entry stating the rule with its cells (or, if
      Excel reads empties as 0, the entry says so and no engine change is made).
- [x] No new F line in any eval/baseline file; `make check` green; reviewers approve.

## Out of scope
- An empty `weekend` argument of WORKDAY.INTL and NETWORKDAYS.INTL (T-2591).
- The financial ToolPak functions (functions-financial is busy; file a follow-up there if the rule holds).
- Teaching the generator to write empty arguments (file a research or harness follow-up if the rule holds).

## Outcome
- **Finding:** in the ToolPak date functions (EDATE, EOMONTH, WORKDAY, NETWORKDAYS, WEEKNUM, YEARFRAC, WORKDAY.INTL,
  NETWORKDAYS.INTL), a written-empty required argument is `#N/A`. The same holds for a LET or LAMBDA variable bound
  to nothing. A written-empty optional argument (holidays, return_type, basis) is treated as omitted.
- **Round 1:** probes/t4870-empty-date-args, 31 cells, excel-graph, 2026-11-07. Predictions were committed in bcbe3f155
  before the fill:

  | hypothesis | cells |
  |---|---|
  | H_na_req: required empty is `#N/A`, optional as today | 26/31 |
  | H_na_all: any empty is `#N/A` | 25/31 |
  | H_zero: the engine before this task | 3/31 |

  The required cells all matched H_na_req. No hypothesis predicted the optional cells, which act as omitted.
- **Round 2:** probes/t4870-r2-empty-optional, 14 fresh calls including the LET and LAMBDA paths. It tested
  "an optional empty argument is omitted", with H_omitted committed in bf24d2c55 before its fill: 14/14.
- **No fit:** each cell is `#N/A` or the omitted call's value, and nothing numeric is tuned.
- **Code:**
  - `toolpak_empty_required` (functions/util.rs) is called by each function's per-element reader, so lifting is
    unchanged. T-4872 and T-4873 can reuse it.
  - `Model::is_written_empty` (logical/let.rs) is the one definition of an empty argument node or variable bound to
    nothing. `complex_pair_of` now uses it, and the holidays readers of WORKDAY(.INTL) and NETWORKDAYS(.INTL) use it to
    treat an empty holidays as omitted.
  - WEEKNUM's reader has a `[serial, EmptyArg]` arm, so an empty return_type means type 1.
  - Unit test: test/test_toolpak_empty_date_args.rs.
- **Docs:** dates.md "A written-empty argument of the ToolPak date functions: required is `#N/A`, optional is omitted
  (T-4870)".
- **Unprobed:** Excel's order between this `#N/A` and another argument's error. The engine checks the empty argument
  first.
- **Eval:** 2 new cases, 45/45.
- **Follow-ups:** none filed. The financial ToolPak functions and the generator are named in the task's Out of scope.
