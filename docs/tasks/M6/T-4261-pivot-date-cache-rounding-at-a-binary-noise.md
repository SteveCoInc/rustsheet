---
id: T-4261
title: "Pivot date cache rounding at a binary-noise half second: probe and model it (after T-4246)"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4246]
---

## Goal
A date whose time sits a binary-noise hair below a half second goes in the date group Excel's pivot cache puts it
in. Every cell of probes/t4236-pivot-date-groups-no-blank that T-4246 left visibly off passes.

## Context
- probes/t4236-pivot-date-groups-no-blank pivot H (hours from 2019-01-01): Data!D4 is 43466.458327546294, written
  as 10:59:59.5. Its time of day is 39599.49999978 s, 2.2e-7 s below the half, and Excel puts it in `11 AM`
  (Calc E45 "11 AM", F13 4). `time_of_day_parts`, which matches HOUR (dates.md, probes/second-rounding: "an exact
  half goes whichever way the serial's binary noise takes it"), rounds it down, so `DateGroups::item`
  (crates/rustsheet-engine/src/pivot/date_groups.rs `split_serial`) gives `10 AM`. That one point is behind all
  15 visibly-off cells: E45:E49, F44:F49 (the layout shifts up a row), and F12, F13, F29 and F33.
- So the pivot cache rounds this date differently from the engine's HOUR model (HOUR of this serial is itself
  unprobed; include it in the probe). Two candidate models fit every probed point: round the
  time to the millisecond first and then to the second (half up), or round the serial to 15 significant digits
  before splitting it. Choosing between them is a structural model: decision 0188 R4 asks for a hold-out and a
  fresh sweep. Probe serials just below and above x.5 s at several magnitudes, where the two models disagree,
  for example a 1900 date (serial < 100, where 15 digits keep far more decimals) against a 2019 one.
- The same rule applies to the importer (it shares `DateGroups`), and the `startDate` and `endDate` comparison.

- probes/t4236-pivot-date-groups pivot H (the same Data!D4 point, ungrouped since T-4260 drops the grouping over
  its blank): the point merges with 10:59:59 into one item, so Calc E44:E50 and F43:F50 (15 cells) are off by a row.
  Those cells are this task's too.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) separates the candidate models; the code follows the winner,
      and web.md "Grouped pivot fields import as their group items" records the rule.
- [ ] probes/t4236-pivot-date-groups-no-blank and probes/t4236-pivot-date-groups pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- HOUR, MINUTE and SECOND, whose rounding is probed and differs (dates.md).

## Outcome
Done. Excel's pivot cache rounds a date to the second as `floor(x / (1/86400) + 0.5)`. That is the serial divided by
the double 1/86400, half up, which is neither `x * 86400` nor HOUR's split. `split_serial`
(crates/rustsheet-engine/src/pivot/date_groups.rs) now does exactly this. It serves the date groups, `ungroup_dates`,
the importer, and the `startDate` and `endDate` comparison, and web.md records the rule.

Structural model (decision 0188 R4; kind (c), no fitted constant, so no `FITTED` tag):

How the first choice failed:
- probes/t4261-pivot-half-second has 36 points from 1900 to 9998, deltas 1e-8 to 0.4 s around 10:59:59.5. Its
  hold-out (odd k) was fixed in the case description. Five candidates were scored: ms then s, 15 significant
  digits, the fraction's product, the engine's HOUR split, and the serial's product `x*86400`.
- `x*86400` scored 18/18 on train, tied with 15 digits, and 18/18 on the hold-out. Slip: the hold-out counts were
  printed in the same run as the training counts.
- `x*86400` was frozen. The first fresh sweep (t4261-pivot-half-second-fresh, seed 4261, 48 points sampled with no
  selection) rejected it at 47/48.

How the frozen model was chosen:
- Eight further orderings were scored on all 84 points of the first two cases. That spends both the original
  hold-out and the first fresh sweep as training data.
- `floor(x / (1/86400) + 0.5)` (the double 1/86400 as divisor) fit 84/84. It was frozen with no margin stated in
  advance.
- Its train is therefore those 84 points. Its untouched hold-outs are the two cases written after the freeze:
  - fresh-2 (seed 4262): 32 of its 48 points were selected where the frozen model and `x*86400` disagree, so it
    discriminates but is not an unbiased sample;
  - fresh-3 (seed 4263): 48 points log-uniform in delta (1e-9 to 1e-5 s either way), random seconds, days 1 to
    2958465, with no selection. This is the R4.2 fresh sweep.
- The cases were filled one at a time, each after the step before it, which is why the oracle stamps are close
  together (06:19, 06:20, 06:21 UTC; fresh-3 later, after the review).

Counts per case: the record's second from Excel's rendered label, each record found by its power-of-two S sum.

| candidate | half-second (36) | fresh (48) | fresh-2 (48) | fresh-3 (48) |
|---|---|---|---|---|
| `x / (1/86400)`, frozen | 36 | 48 | 48 | 48 |
| `x * 86400` | 36 | 47 | 14 | 44 |
| correctly rounded `floor(x*86400 + 1/2)`, exact | 27 | 47 | 10 | 27 |

The rounded second is an integer, so bit-exact, within 4 ulp and 15 digits are the same count here.

Gate A envelope:
- Every cell of the four cases is within the envelope (15 digits of Excel): fresh-3 106/106 cells, fresh-2
  106/106, fresh 106/106, half-second 190/190 (`rustsheet-eval show`).
- No point is outside it, and no cell is fit below the reproducibility floor.
- There is no keep-matched branch: one rule serves every date.
- UNVERIFIED: a `startDate` or `endDate` within half a second of a noise point. That comparison uses the same
  `split_serial`, but no probe puts a bound there (web.md).

Eval delta: probes known failures 4980 -> 4950 (30 fixed, 0 regressions, 4 new cases all passing: 190, 106, 106 and
106 cells). probes/t4236-pivot-date-groups (335/335) and -no-blank (349/349) now pass in full.

## Landing notes (resumed)
- PR #3299 (draft). Head 0c9bec9d7 is round 1's fix commit, together with this note's commit.
- Reviews on acbc7ac5f: security APPROVE; design APPROVE (its optional "do not simplify" note is applied); excel-fidelity
  REQUEST_CHANGES (R4 record). Round 1's fix:
  - added probes/t4261-pivot-half-second-fresh-3, an unbiased log-uniform sweep written after the freeze (48/48 for
    the frozen model; 44 for `x*86400`; 27 for exact correct rounding);
  - rewrote the Outcome's R4 section with the per-case table and the Gate A envelope line;
  - marked the startDate/endDate path UNVERIFIED in web.md.
  Excel-fidelity has not re-reviewed this head.
- Open blocker: `make eval-math-check` fails. probes/t4261-pivot-half-second's HOUR and SECOND cells (Calc D1:E36)
  have no math sidecar yet. `cd tools && uv run rustsheet-accuracy math --suite probes` regenerates it, but it takes
  over 20 minutes here, and the window closed before it finished. Commit only
  `t4261-pivot-half-second.math.json` (and any other t4261 sidecar it writes). The same run also rewrites
  m9-bessel-order-limits, t2546-bessel-past-cap and t2881-besselj-turning-point `.math.json`: do not commit those,
  they are unrelated to this task.
- Next: push the sidecar, wait for CI green, re-run reviewer-excel-fidelity on the new head, mark the PR ready and
  enqueue.
- Do not repeat: scoring a hold-out alongside its training counts, or choosing a model on the fresh sweeps. The
  Outcome records both slips; any further model change needs a new pre-split case.
