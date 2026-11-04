# T-3580: F on Excel's r >= 1 arguments with bgrat at b = 1/2

Model (structural, reviewer-excel-fidelity check 7 (c)): T-3567's argument form (statistics.md "F's incomplete-beta
arguments keep r >= 1") in `fisher.rs` `f_beta_arguments`, and F's tails through `bratio::f_closed_or_ported_tails`
(bgrat at b = 1/2 and L140 bup+bgrat, as the Student t and BETA.DIST take them). Frozen at 3f4e01a2; the fresh sweep was
drawn and filled after that commit and scored only once the per-branch check below had left the model unchanged.

## Hold-out: probes-numeric t3567-r5-fdf1-hold (T-3567's, 3,578 cells)

Bit-exact / within 4 ulp / within 15 significant digits (`score.py t3567-r5-fdf1-hold`):

| region | cells | T-3580 | main | correctly rounded (T-3567) |
|---|---|---|---|---|
| bg1: d1 = 1, d2 31-400, bgrat | 1,000 | 991/1,000/1,000 | 298/572/659 | 311/612 |
| d1: d1 = 1, d2 3-30 | 800 | 634/794/799 | 594/766/784 | 298/631 |
| d2: d2 = 1 | 600 | 452/595/599 | 346/522/536 | 283/416 |
| gen | 1,000 | 652/864/911 | 596/813/872 | 432/673 |
| tie: m·x = n | 178 | 150/177/178 | 150/177/178 | 83/152 |
| all | 3,578 | **2,879/3,430/3,487** | 1,984/2,850/3,029 | 1,407/2,484 |

No cell goes from within 15 digits to visibly off; 458 go the other way. T-3567's research model scored 2,787/3,419/3,463.

Per branch (the branch of each cell on the new arguments; T-3580 against the alternative named), bit-exact / within 4 ulp:

| branch | cells | T-3580 | alternative |
|---|---|---|---|
| Bgrat | 1,192 | 1,177/1,191 | old arguments 840/903 |
| BupBgrat | 98 | 65/98 | old arguments 48/75; today's kernel about 31 |
| L140 bup+bpser | 294 | 220/293 | old arguments 204/270 |
| L140 bup+bgrat | 52 | 29/47 | old arguments 28/45; off this branch (today's kernel) the whole hold-out has 14 fewer bit-exact cells |
| W1 bpser | 132 | 108/132 | old arguments 98/122 |
| Bfrac | 210 | 103/146 | old arguments 103/143 |
| WBpser, BpserLarge | 1,596 | 1,176/1,519 | the same (they read x only) |

So the new form is kept in every branch (the task's "keep whichever is closer per branch").

## Gated suites

gen-m9 fixes 18, probes 20. Three gated cells regress by 1-2 ulp, all in bup (decision 0263): `FDIST(2,1,9)` twice
(bup+bgrat at b = 1/2; T-3567 found its research kernel 1 ulp off there too) and `F.DIST.RT(1,7,9)` (L140 bup+bpser).

## Fresh sweep: probes-numeric t3580-f-fresh (`mkfresh.py`, seed 35800, 2,400 cells, excel-graph)

| region | cells | T-3580 | main | correctly rounded |
|---|---|---|---|---|
| bg1 | 800 | 712/792/800 | 252/522/574 | |
| d1 | 600 | 476/591/599 | 445/570/590 | |
| d2 | 400 | 278/390/394 | 210/345/351 | |
| gen | 600 | 401/507/542 | 371/487/525 | |
| all | 2,400 | **1,867/2,280/2,335** | 1,278/1,924/2,040 | 932/1,718/1,872 |

0 cells newly visibly off against main. Gate A envelope (`gatea.py`, math from `fmath.py`, attached as
`t3580-f-fresh.math50.json`; the probe's tool sidecar covers only the F.DIST column, and agrees with it to 4e-40):
2,400 of 2,400 inside, none outside to list. The Gate A batch `eval/accuracy/tables/F.DIST/r4-T-3580` (196 points,
drawn and filled after the freeze) is all inside: `rustsheet-eval accuracy` 13,334 points, 0 failing, 0 table defects.

R4.4: the gated losses are 1-2 ulp, below the 4-ulp web-against-desktop floor; nothing here is tuned to them.
