# T-4635: GAMMA.DIST at a huge finite x / beta

`gen.py` builds probes/t4635-gamma-dist-huge-ratio. It and these predictions were committed before the excel-graph
fill.

Evidence before the fill (probes/t4625-gamma-dist-ratio-overflow): at x / beta = 1E308, GAMMA.DIST is `#NUM!` at
shapes 1.5 and 2 in both forms, and a number at shapes 0.5, 1, 2.5, 2.9, 3, 10 and 1E5. At shape 2 it is a number at
x / beta 1E200 and below.

Hypothesis H, predicted before the fill:
- `#NUM!` exactly for 1 < a < 2.5 at x / beta 1E308, and a number for a = 1 and a >= 2.5.
- Every shape is a number at x / beta 1E200.
- CHISQ.DIST and CHIDIST at df 2 to 5, x 1E200 and 1E290 (Excel for the web refuses a workbook with x 1E307 or more, first
  fill), show what the shared incomplete gamma does below the cut; a `#NUM!` there would be
  comes from the shared incomplete gamma.

The ratio cut is unknown. The 1E250 to 1.7E308 columns locate it. Nothing is tuned to the fill: the rule recorded is
what the cells show, and a cut is implemented only where the probe brackets it.

Run: `python3 -I docs/research/t4635/gen.py eval/suites/probes/cases/t4635-gamma-dist-huge-ratio.case.json`

## First fill (excel-graph 2026-11-07) and the second probe

Note: the first probe's columns are x / beta 1E216, 1E258, 1E298, 1E306, 1E307, 1E308, 1.5E308 and 1.7E308 (x times
1E8), not "1E200 and up" as the generator's comment says.

What the first fill shows (cdf and density alike, every cell): shape 1 is a number everywhere; shape 1.01 is `#NUM!`
from 1E306, shape 1.1 from 1E298; shapes 1.25 to 2.1 from 1E308; shapes 2.25 to 2.51 from 1.5E308. Hypothesis H is
refuted (shapes 2.25 to 2.51 fail at 1.5E308; 1.01 and 1.1 fail below 1E308). CHISQ.DIST and CHIDIST at df 2 to 5 are
numbers at 1E200 and 1E290.

Hypothesis H2, committed before the second fill (`gen2.py`, probes/t4635-gamma-dist-huge-ratio-cut):
- Low regime, shapes just above 1: `#NUM!` exactly where y^a overflows a double (y = x / beta), which fits 1.01
  (cut 1E305.2) and 1.1 (cut 1E280.2). It cannot hold at 1.25 (1E258^1.25 overflows, a number), so it stops at a
  shape between 1.1 and 1.25.
- High regime: `#NUM!` where y * (4 - a) overflows, for 1 < a < 4 (fits every first-fill cell from 1.25 to 2.51,
  and 2.9 and 3 at 1E308 from T-4625). Shapes 0.5 and 1 do not follow it (numbers at 1E308).
- The cut is on x / beta, not on x: the Beta sheet repeats the first fill's verdicts.
