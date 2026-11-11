# Probe request: LINEST's zero-head rule, the fresh evidence decision 0188 R4.2 asks for (T-2574, poc-gateb-cc)

Unblocks CR-321684429 (branch `gateb/linest-zero-head`). Both reviewers block on R4.2: the rule was chosen by scoring
candidates against an Excel cell, so it owes Excel values drawn after it was frozen. Nothing here was read to find it.

## What to fill (excel-graph, raw values)

Five cases, at their suite paths under this directory. Copy `eval/` over a checkout of main (or of the branch):

| Case | Checked cells | What it is |
|---|---:|---|
| `eval/suites/probes/cases/t2574-zero-head-fresh.case.json` | 61 | The aimed probe: 55 least-squares formulas on sheet P (column B) whose QR meets a pivot head of exactly 0, plus the Precision canary |
| `eval/accuracy/tables/LINEST/r4-T-2574.case.json` | 200 | The R4.2 sweep: a fresh full-domain batch from the standard generator (`rustsheet-accuracy generate LINEST --batch r4-T-2574`) |
| `eval/accuracy/tables/TREND/r4-T-2574.case.json` | 200 | the same for TREND |
| `eval/accuracy/tables/LOGEST/r4-T-2574.case.json` | 200 | the same for LOGEST |
| `eval/accuracy/tables/GROWTH/r4-T-2574.case.json` | 200 | the same for GROWTH |

The four batches carry their math sidecars (`r4-T-2574.math.json`); the fill adds `r4-T-2574.oracle.json` beside each.

## Commands

From `tools/`:

```bash
uv run rustsheet-oracle fill -b excel-graph --suite probes --case probes/t2574-zero-head-fresh.case.json
uv run rustsheet-oracle fill -b excel-graph --suite accuracy
```

The second command fills every pending batch of the accuracy suite; on main that is only these four.

In the probe workbook `Precision!B2:B4` must read 0.9999999999999998, 1, 1.0000000000000002 and `Precision!C2:C4`
-1, 0, 1. A canary failure means stop.

## What comes back

- `eval/suites/probes/cases/t2574-zero-head-fresh.oracle.json`
- `eval/accuracy/tables/{LINEST,TREND,LOGEST,GROWTH}/r4-T-2574.oracle.json`

## What the aimed probe decides

`docs/research/t2574/zero-head-fresh-predictions.tsv` holds the engine's value for each of the 55 formulas under
both rules (hypotheses, never expected values): the old rule (main: a pivot head of exactly 0 reflects as a positive
one) and the new rule (T-2574: as a negative one). 42 of the 55 differ between the rules; 13 agree and are controls.

The cells that matter most are group Z, where one rule gives an exact 0 and the other gives noise:

| Cell | Formula | Old rule | New rule |
|---|---|---|---|
| P!B2 | `=INDEX(LINEST({1,3,0,2},{1,2,3,4}),1,1)` | -0 | -9.930136612989092E-17 |
| P!B3 | `=INDEX(LINEST({3,0,6,1},{1,2,3,4}),1,1)` | -1.9860273225978183E-16 | 0 |
| P!B5 | `=INDEX(LINEST({0,1,1,1},{1,2,3,4}),1,2)` | -1.1102230246251565E-16 | 0 |
| P!B6 | `=INDEX(LINEST({0,0,3,0},{1,2,3,4}),1,2)` | 0 | -1.1102230246251565E-16 |

- **If Excel equals the new rule on the 42:** the rule holds on shapes it was not found on; the CR goes ahead.
- **If Excel equals the old rule on some:** the rule is wrong for those shapes and the CR does not land as written.
- **If Excel equals neither:** the kernel is missing something else there; the cells become T-2574's next evidence.

The groups (column A of sheet P): Z zero against noise over x = 1..4 with a constant; L last digits of the same shape;
G LOGEST and GROWTH; N no constant with a first x of 0; M the first two x at the mean; T two predictors without a
constant; S other steps and orders of x; R every statistic of one fit; D neighbours of the deciding fit; C the other
callers; P two predictors with a constant; K controls with no zero head.

24 of the formulas are reviewer 1's (CR-321684429 round 1); the rest are mine.

## The sweep also discriminates

On 7 of the 800 points of the four `r4-T-2574` batches the two rules give different engine values (measured with
both kernels, `rustsheet-eval accuracy --json`):

| Point | Formula (shortened) | Old rule | New rule | Math |
|---|---|---|---|---|
| LINEST A118 | `INDEX(LINEST({24,19,7,10},{1,2,3,4},TRUE,FALSE),1,1)` | -5.4 | -5.3999999999999995 | -5.4 |
| LINEST A140 | `INDEX(LINEST({1,9,5,7},{0,4,2,3},FALSE,FALSE),1,1)` | 2.3103448275862064 | 2.3103448275862073 | 2.310344827586207 |
| LOGEST A51 | a four-point LOGEST | 2.9636461069010513 | 2.9636461069010505 | 2.963646106901051 |
| LOGEST A190 | a LOGEST over 48 orders of magnitude | 48.11222011094902 | 48.1122201109484 | 48.11222011094851 |
| TREND A72 | a four-point TREND | 88240.20039601852 | 88240.20039601851 | 88240.2003960185 |
| GROWTH A21 | a four-point GROWTH | 13829.500706432911 | 13829.500706432937 | 13829.500706432937 |
| GROWTH A72 | a four-point GROWTH | 0.8077828810955523 | 0.8077828810955524 | 0.8077828810955524 |

LINEST A118 is the sharp one: the old rule gives the exact -5.4 and the new rule is one unit in the last place off.

## Until the fill lands, the batches are not committed

`rustsheet-eval accuracy` with the four unfilled batches reports 800 pending points and **4 failing**: GROWTH A194 and
A48, LINEST A182 and A141. They are ill-conditioned draws where the engine is more than 1e-9 from the 50-digit math, and
without an Excel value nothing else can admit them. They fail identically under main's kernel, so they are not this
change's. Once filled, each is either inside the envelope on Excel's value or a known failure of the batch with an
owner task. So the batches go into the CR together with their oracle sidecars, not before.
