# Probe request: fresh cells for the paired T.TEST rule (decision 0188 R4.2; T-4579, poc-gateb-cc)

One case, 384 checked cells, excel-graph, raw values (enlarged 2026-11-11 22:3xZ in review of CR-321848156, before any fill:
the first five sheets are unchanged, two sheets were added):
`eval/suites/probes/cases/t4579-ttest-paired-fresh.case.json`, copied here at its suite path. It is not on main yet;
a CR for the probe alone follows (branch `gateb/ttest-paired-probe`).

```bash
# from tools/
uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/t4579-ttest-paired-fresh.case.json
```

What comes back: `eval/suites/probes/cases/t4579-ttest-paired-fresh.oracle.json`.

## Why

T.TEST fails the expert tier gate (decision 0355) on one probe, probes/t4579-ttest-paired: 30 cells of a wrong
type and 15 outside the envelope, of 345 (3 allowed). The task had ended with "Excel's algorithm is not
identified". It is now: the variance of the differences is `vx - 2 cov + vy`, the samples' variances and
covariance each from its own two-pass sum in doubles, and `#DIV/0!` where that is 0 or below. On the first probe
it gives Excel's error on all 33 error cells and Excel's number within 1E-12 on all 18; on a hold-out, Gate A's
112 paired table points, the same error on all 10 and 15 digits on all 102. It is a structural rule found by
scoring forms on Excel cells, so decision 0188 R4.2 owes it Excel values drawn after it was frozen. The engine
change is ready on branch `gateb/ttest-paired` and waits for this fill.

## What is in it

Every formula carries its own literals (15 significant digits at most; the generator asserts each reads back as the
double the prediction used). Strata, none chosen by Excel's answer:

| Sheet | Cells | What it is | The rule predicts |
|---|---:|---|---|
| Near | 60 | near-constant differences, 2 to 8 pairs, magnitudes 1E-3 to 1E6 | 47 `#DIV/0!`, 13 numbers |
| Offset | 40 | both samples on large offsets with a shared spread, 4 to 12 pairs | 24 `#DIV/0!`, 16 numbers |
| Binade | 40 | a sample variance that is a power of two, kept where rounding `vx + vy` first would change the answer | 40 numbers |
| Plain | 30 | unrelated or loosely related samples, 2 to 30 pairs | 30 numbers |
| Edge | 24 | identical samples, exactly constant differences, squares that overflow or underflow, one tail, TTEST | 5 `#DIV/0!`, 6 `#NUM!`, 13 numbers |
| Straddle | 40 | the two variances on opposite sides of a power of two, kept where the three orders of the terms disagree, each row in both argument orders | 12 `#DIV/0!`, 28 numbers |
| Last | 150 | more plain samples, the hold-out for how t is formed | 150 numbers |

Frozen before the fill, in `docs/research/t4579/` here: `fresh-predictions.tsv` (per cell: vx, vy, cov, the variance
under the rule and under three other orders of the terms, and each one's p from a 60-digit tail) and
`fresh-engine.tsv` (the values of the engine with the rule).

## What the fill decides

- Near, Offset, Binade: whether Excel is `#DIV/0!` exactly where the rule's variance is 0 or below, and the rule's
  number elsewhere. A different answer on any of these 140 cells refutes the rule.
- Straddle: which order of the three terms is Excel's. The other two orders differ from the rule there on 20 and
  on all 40 cells (12 and 24 of them an error against a number).
- Last: how t is formed, by cells bit-exact with `fresh-engine.tsv`.
- Edge: three things the first probe did not hold. A sample whose squares overflow beside equal differences (the
  rule says `#NUM!`); products that underflow (VAR flushes a subnormal product to 0; the rule as coded does not);
  exactly constant differences whose samples' deviations round differently (`T.TEST({0.5,0.25,0.125},
  {1.5,1.25,1.125},2,1)`: the rule says a number, 2.3E-18, where the differences are exactly constant).

No urgency beyond the next batch. It unblocks the engine change for T-4579 (T.TEST and TTEST leave the expert
tier's failing list).
