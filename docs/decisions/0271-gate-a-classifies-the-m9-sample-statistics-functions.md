# 0271 — Gate A classifies the M9 sample-statistics functions with no table

- **Status:** accepted (2026-11-04, T-3799). Amends `eval/accuracy/exempt.tsv`, a protected path (decision 0199).
- **Context:**
  - Gate A (decision 0188 R1(a), SPEC-0029 "Gate A holds") needs every in-scope numeric catalog function to have a
    reference table or an `exempt.tsv` line. Decisions 0228 (M5 to M7) and 0270 (M9 engineering and database) set the
    pattern.
  - T-3799 listed 74 M9 functions that compute a statistic over sample data. A recount after decision 0270
    (`missing tables 182`) found all 74 still with neither.
  - The dependencies T-3799 names are done: T-3549 (variadic draws), T-3686 (2-D literals), T-3529 (chi-squared).
- **Decision:** each of the 74 functions is classified below.
  - **`exact`, 8 functions.** The result is one of the inputs or a count, with no floating-point step:
    - MAXA and MINA: over an array they read only its numbers, and they are 0 with none (statistics.md "AVERAGEA, MAXA
      and MINA over an array");
    - LARGE and SMALL: the ceil(k)-th largest or floor(k)-th smallest input (statistics.md "LARGE/SMALL: a fractional
      k rounds up for LARGE and down for SMALL");
    - MODE, MODE.SNGL and MODE.MULT: the most frequent inputs. MODE.MULT returns them as an array;
    - FREQUENCY: an array of counts, one per bin plus one (statistics.md "FREQUENCY: one row per bin plus one").
  - **`composite`, 11 functions:**
    - AVERAGEIF, AVERAGEIFS, MAXIFS and MINIFS: their ranges must be references, as for SUMIF and SUMIFS (decision
      0217, T-3550);
    - RANK, RANK.EQ and RANK.AVG: `ref` must be a reference. Excel for the web refuses a workbook with
      `RANK(5,{7,1,5})` (statistics.md "RANK of a non-reference array");
    - FORECAST.ETS, FORECAST.ETS.CONFINT, FORECAST.ETS.SEASONALITY and FORECAST.ETS.STAT: Excel for the web returns
      `#N/A` for every valid call (statistics.md "FORECAST.ETS family: excel-graph never computes it"), so an
      excel-graph table holds no number. The algorithm has no closed form to tabulate against either. T-2425 (M14,
      desktop evidence) owns the implementation; when it lands, a table task can replace these lines.

    Each reason names the eval suites that drive the function (gen-m9 and probes for all, and probes-numeric for the
    averages).
  - **Needs a table, 55 functions,** filed as ten table tasks by kernel:
    - T-4089: STDEV, STDEV.S, STDEVA, VAR, VAR.S, VARA, DEVSQ;
    - T-4090: STDEV.P, STDEVP, STDEVPA, VAR.P, VARP, VARPA, AVEDEV;
    - T-4091: KURT, SKEW, SKEW.P, GEOMEAN, HARMEAN, AVERAGEA, TRIMMEAN;
    - T-4092: MEDIAN, PERCENTILE, PERCENTILE.INC, PERCENTILE.EXC, QUARTILE, QUARTILE.INC, QUARTILE.EXC. MEDIAN is not
      `exact`: an even count is a + (b - a)/2 (statistics.md);
    - T-4093: PERCENTRANK, PERCENTRANK.INC, PERCENTRANK.EXC;
    - T-4094: FORECAST, FORECAST.LINEAR, INTERCEPT, SLOPE, RSQ, PEARSON, CORREL;
    - T-4095: COVAR, COVARIANCE.P, COVARIANCE.S, STEYX;
    - T-4096: LINEST, LOGEST, TREND, GROWTH. Each point reads one element through INDEX;
    - T-4097: T.TEST, TTEST, F.TEST, FTEST;
    - T-4098: CHISQ.TEST, CHITEST, Z.TEST, ZTEST, PROB.
- **Consequences:**
  - `exempt.tsv` gains 19 lines (8 `exact`, 11 `composite`). `missing tables` falls from 182 to 163, with no stale or
    bad exemption.
  - The ten table tasks bring this group to zero missing tables once they land.
  - As in decision 0228, an `exact` function that later shows a rounding question loses its line to a table task.
    That needs a new decision record, because the path is protected.
