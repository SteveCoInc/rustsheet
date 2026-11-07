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
