"""Generate the draft rustsheet-case/v1 probe cases (stdlib only; writes JSON, runs no repo code).

Usage: python3 -I gen_cases.py OUTDIR
Each case: one sheet "P", formulas only, "expected": {}. Layout per case is documented in PROBES.md.
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

CASES: dict[str, dict] = {}


def col(n: int) -> str:
    s = ""
    n += 1
    while n:
        n, r = divmod(n - 1, 26)
        s = chr(65 + r) + s
    return s


def lit(x: float) -> str:
    """A literal of at most 15 significant digits."""
    # Excel cuts a formula literal to 15 significant digits (excel-kernel-playbook "Traps"): emit at most 15.
    return ("%.15g" % float(x)).replace("e", "E")


def case(cid: str, desc: str, columns: list[list[str]], tags: list[str] | None = None) -> None:
    cells: dict[str, str] = {}
    for c, formulas in enumerate(columns):
        for r, f in enumerate(formulas):
            if f:
                cells[f"{col(c)}{r + 1}"] = f
    CASES[cid] = {
        "format": "rustsheet-case/v1",
        "description": desc,
        "tags": tags or ["M9", "statistical", "probe"],
        "sheets": [{"name": "P", "cells": cells}],
        "expected": {},
    }


# ---------------------------------------------------------------- G01 CHISQ.INV.RT Q->P switch
ps = [f"0.9999+{k}*2E-6" if k else "0.9999" for k in range(28)] + ["0.9999462", "0.99996", "0.99997", "0.99998",
                                                                    "0.99999"]
ulps = [f"0.9999{'+' if j > 0 else '-'}{abs(j)}*2^-53" for j in (-4, -3, -2, -1, 1, 2, 3, 4)]
cols = []
for df in (3, 5, 4, 7, 10):
    cols.append([f"=CHISQ.INV.RT({p},{df})" for p in ps + ulps])
cols.append([f"=CHIINV({p},3)" for p in ps + ulps])
cols.append([f"=CHISQ.INV(1-({p}),3)" for p in ps])  # rows 1-33 only
case(
    "val-chisq-inv-rt-q-to-p-switch",
    "CHISQ.INV.RT just above p=.9999: where Excel stops inverting the rounded Q and returns to the small-P "
    "solver (known only to lie in (.9999, .9999462]). Columns A:E df 3,5,4,7,10; F CHIINV df 3; G CHISQ.INV(1-p,3). "
    "Rows 1-33: p=.9999+k*2E-6 (k=0..27), .9999462, .99996, .99997, .99998, .99999; rows 34-41: .9999 -4..+4 "
    "ulps (2^-53).",
    cols,
)

# ---------------------------------------------------------------- G02 unit x = .5 boundary
xs = [round(0.80 + 0.02 * i, 2) for i in range(21)] + [0.996, 0.998, 1.002, 1.004]
cols = []
for df in (3, 5, 8, 12, 20):
    cols.append([f"=CHISQ.DIST.RT({x},{df})" for x in xs])
    cols.append([f"=CHISQ.INV.RT({col(len(cols) - 1)}{i + 1},{df})" for i in range(len(xs))])
    cols.append([f"=CHISQ.INV(CHISQ.DIST({x},{df},TRUE),{df})" for x in xs])
case(
    "val-chisq-inv-unit-half-boundary",
    "Gamma/chi-square inverse around unit x = .5 (chi-square x = 1), where the engine stops using the "
    "leading-factor start + Halley iteration and falls back to its accurate crossing. Per df (3,5,8,12,20) three "
    "columns: forward CHISQ.DIST.RT(x), CHISQ.INV.RT of that cell, CHISQ.INV of the lower CDF. Rows: x=.80..1.20 "
    "step .02, then .996,.998,1.002,1.004.",
    cols,
)

# ---------------------------------------------------------------- G03 split complement region
# x points where (0.5-P)+0.5 != 1-P for the engine's P (found by a dense x=.01..6 scan); df is integer (CHISQ.DIST truncates).
SPLIT_X = {"2": [0.04, 0.08, 0.09, 0.11, 0.15], "3": [0.07, 0.08, 0.1, 0.12, 0.14, 0.34, 0.38, 0.64], "4": [0.01, 0.06, 0.12, 0.17, 0.27, 0.49, 0.54, 0.71], "5": [0.06, 0.18, 0.3, 0.47, 0.71, 0.89, 1.1, 1.33], "9": [0.03, 0.25, 0.39, 0.54, 0.83, 1.13, 1.46, 1.62, 2.07, 2.32, 2.58, 2.69, 2.93, 3.31, 3.7, 3.87], "17": [0.1, 0.35, 0.5, 0.75, 0.88, 1.16, 1.44, 1.71, 2.03, 2.48, 2.9, 3.52, 3.93, 4.22, 4.58, 5.11], "39": [2.54, 2.92, 3.33, 3.7, 4.14, 4.5, 5.09, 5.45], "40": [2.73, 3.07, 3.52, 3.99, 4.41, 4.95, 5.29, 5.73], "41": [2.93, 3.1, 3.53, 3.86, 4.33, 4.65, 5.08, 5.4]}
cols = []
for df, xs in SPLIT_X.items():
    cols.append([f"=CHISQ.DIST.RT({x},{df})" for x in xs])
    cols.append([f"=1-CHISQ.DIST({x},{df},TRUE)" for x in xs])
    cols.append([f"=(0.5-CHISQ.DIST({x},{df},TRUE))+0.5" for x in xs])
case(
    "val-gamma-split-complement-region",
    "Where Excel's upper gamma tail is the split complement (0.5-P)+0.5 rather than 1-P. The engine uses the split "
    "form only for shape a=df/2 > 1 at unit x = x/2 <= 1 (chi-square x <= 2). Every row is a point where the two "
    "forms differ. Per df (2,3,4,5,9,17,39,40,41) three columns: CHISQ.DIST.RT, 1-P, (0.5-P)+0.5; the x list per df "
    "is in PROBES.md. Rows with x > 2 and df 2 (a = 1) are outside the engine's split region.",
    cols,
)

# ---------------------------------------------------------------- G04 small-x prefactor boundary
us = ["1-4*2^-53", "1-2^-53", "1", "1+2^-52", "1+4*2^-52", "0.999", "1.001", "0.5", "1E-3"]
shapes = ["1+2^-52", "1.0001", "2", "8", "30", "100", "170", "170.5", "171", "172"]
cols = []
for a in shapes:
    cols.append([f"=GAMMA.DIST({u},{a},1,TRUE)" for u in us] + [f"=GAMMA.DIST(2*({u}),{a},2,TRUE)" for u in us])
    cols.append([f"=GAMMA.DIST({u},{a},1,FALSE)" for u in us] + [f"=GAMMA.DIST(2*({u}),{a},2,FALSE)" for u in us])
case(
    "val-gamma-small-x-prefactor-boundary",
    "T-3219's stored a*LN(x/beta) / EXP(GAMMALN(a+1)) small-x prefactor is used for shape > 1 and x/beta <= 1. "
    "Probe both edges: x/beta = 1 +-ulps and shapes just above 1 and around 170-172 (EXP(GAMMALN(a+1)) overflow). "
    "Per shape two columns (CDF, density); rows 1-9 beta=1, rows 10-18 the same x/beta with beta=2.",
    cols,
)

# ---------------------------------------------------------------- G05 far upper tail reciprocal log
cols = []
for a in (1.5, 2.5, 6, 15, 40, 100):
    cols.append([f"=CHISQ.DIST.RT({2 * a * r},{2 * a})" for r in (3, 4, 6, 8, 9.5, 10, 10.5, 12, 16, 20, 30)])
    cols.append([f"=POISSON.DIST({int(a) - 1},{a * r},TRUE)" if a == int(a) else ""
                 for r in (3, 4, 6, 8, 9.5, 10, 10.5, 12, 16, 20, 30)])
case(
    "val-gamma-far-upper-tail-shapes",
    "T-3469 reciprocal-log far upper tail B = a*LN(a/x)+(x-a) at shapes other than the shape-6 evidence. Per shape "
    "(1.5,2.5,6,15,40,100): CHISQ.DIST.RT(2*a*r, 2a) and a POISSON CDF companion (integer shape only meaningful); "
    "rows x/a = 3,4,6,8,9.5,10,10.5,12,16,20,30.",
    cols,
)

# ---------------------------------------------------------------- G06 AS109 accuracy exponent
ms = ["1", "1.1", "1.118", "1.12", "1.25", "1.29", "1.3", "1.58", "1.59", "2.23", "2.24", "3"]
pv = ["0.001", "0.0041", "0.0042", "0.01", "0.03", "0.03125", "0.032", "0.1", "0.3", "0.5", "0.7", "0.96875", "0.99"]
cols = []
for other in ("3", "10"):
    for m in ms:
        cols.append([f"=BETA.INV({p},{m},{other})" for p in pv])
        cols.append([f"=BETA.INV({p},{other},{m})" for p in pv])
case(
    "val-beta-inv-as109-accuracy-exponent",
    "BETA.INV's AS 109 stop with accuracy 10^(-5/min(a,b)^2 - p^(-0.2) - 13), untruncated. The small shape m crosses "
    "the integer points of -5/m^2 (m=1, 1.118, 1.29, 1.58, 2.236) and p crosses p^-0.2 = 2,3 (p=1/32, .00412) and their "
    "mirrors. Column pairs (m,other) then (other,m), other=3 for the first 24 columns and 10 for the next 24; "
    "rows p = .001,.0041,.0042,.01,.03,.03125,.032,.1,.3,.5,.7,.96875,.99.",
    cols,
)

# ---------------------------------------------------------------- G07 endpoint replacement / saturation
shp = [(0.5, 0.5), (1, 1), (1, 3), (3, 1), (0.1, 10), (10, 0.1), (5, 5), (50, 50), (0.5, 50), (50, 0.5), (2, 5e9)]
pp = ["1E-20", "1E-50", "1E-100", "1E-200", "1E-300", "1-1E-12", "1-1E-15", "1-2^-52"]
cols = [[f"=BETA.INV({p},{a},{b})" for p in pp] for a, b in shp]
for d1 in (1, 2):
    for d2 in (1, 2, 3, 5, 10, 30):
        cols.append([f"=F.INV({p},{d1},{d2})" for p in pp])
        cols.append([f"=F.INV.RT({p},{d1},{d2})" for p in pp])
case(
    "val-beta-f-inv-endpoint-saturation",
    "AS 109 endpoint replacement (start outside (0,1) -> 1E-6 / 1-1E-6) and the damped far-tail trajectory across "
    "shape regions. Columns A:K BETA.INV at shapes (.5,.5),(1,1),(1,3),(3,1),(.1,10),(10,.1),(5,5),(50,50),(.5,50),"
    "(50,.5),(2,5E9); then F.INV / F.INV.RT pairs for df1 in {1,2} x df2 in {1,2,3,5,10,30}. Rows p = 1E-20,1E-50,"
    "1E-100,1E-200,1E-300,1-1E-12,1-1E-15,1-2^-52.",
    cols,
)

# ---------------------------------------------------------------- G08 F back-transform
dd = [(1, 1), (2, 3), (5, 5), (4, 10), (10, 4), (30, 7), (1, 100), (100, 1)]
pp = ["1E-10", "0.01", "0.3", "0.5", "0.9", "0.999"]
cols = []
for d1, d2 in dd:
    cols.append([f"=F.INV({p},{d1},{d2})" for p in pp] + [f"=F.INV.RT({p},{d1},{d2})" for p in pp])
    bcol = col(len(cols))
    cols.append([f"=BETA.INV({p},{d1}/2,{d2}/2)" for p in pp] + [f"=BETA.INV(1-({p}),{d1}/2,{d2}/2)" for p in pp])
    cols.append([f"={d2}*{bcol}{i + 1}/({d1}*(1-{bcol}{i + 1}))" for i in range(12)])
    cols.append([f"=EXP(LN({bcol}{i + 1})-LN(1-{bcol}{i + 1})+LN({d2})-LN({d1}))" for i in range(12)])
case(
    "val-f-inv-back-transform",
    "F inverse from the beta quantile: Excel's back-transform (compensated four-log ratio "
    "LN(df2)+LN(x)-LN(df1)-LN(1-x)) vs the direct quotient. Per df pair (1,1),(2,3),(5,5),(4,10),(10,4),(30,7),(1,100),"
    "(100,1) four columns: F.INV/F.INV.RT, BETA.INV of the same tail, direct quotient of that beta, worksheet four-log "
    "form. Rows 1-6 F.INV at p=1E-10,.01,.3,.5,.9,.999; rows 7-12 F.INV.RT at the same p (beta at 1-p).",
    cols,
)

# ---------------------------------------------------------------- G09 F tiny-tail neighbourhood
cols = []
for d1 in (7, 8, 9, 10, 11):
    cols.append([f"=F.INV(1-1E-16,{d1},{d2})" for d2 in (3, 4, 5, 6, 7)] +
                [f"=F.INV(1E-300,{d1},{d2})" for d2 in (2, 3, 4, 5)] +
                [f"=F.INV(1E-250,{d1},{d2})" for d2 in (2, 3, 4, 5)] +
                [f"=F.INV.RT(1E-16,{d1},{d2})" for d2 in (3, 4, 5, 6, 7)])
case(
    "val-f-inv-tiny-tail-neighbourhood",
    "Neighbours of the two remaining F.INV misses (F.INV(1-1E-16,9,5), F.INV(1E-300,9,3)) to pin the small-tail "
    "beta-series dispatch and the subnormal-residual stop. Columns df1=7..11; rows 1-5 F.INV(1-1E-16,df1,3..7), "
    "6-9 F.INV(1E-300,df1,2..5), 10-13 F.INV(1E-250,df1,2..5), 14-18 F.INV.RT(1E-16,df1,3..7).",
    cols,
)

# ---------------------------------------------------------------- G10 F AS109 tolerance cap
pp = ["0.3", "0.35", "0.4", "0.45", "0.55", "0.6", "0.7"]
cols = [[f"=F.INV({p},{k},{k})" for p in pp] + [f"=FINV({p},{k},{k})" for p in pp] for k in range(2, 13)]
cols += [[f"=F.INV({p},{k},{k + 3})" for p in pp] for k in (3, 5, 8)]
case(
    "val-f-inv-as109-tolerance-cap",
    "F's AS 109 absolute tolerance capped at EPSILON^2 plus a unit-roundoff relative stop (fixes F.INV(.4,5,5)). "
    "Columns A:K df (k,k) k=2..12 (rows 1-7 F.INV, 8-14 FINV right tail at p=.3,.35,.4,.45,.55,.6,.7); L:N F.INV at "
    "(3,6),(5,8),(8,11).",
    cols,
)

# ---------------------------------------------------------------- G11 Student-t joint stop
dfs = list(range(2, 31)) + list(range(123456785, 123456796))
cols = [[f"=T.INV.2T(0.5,{d})" for d in dfs]]
for p in ("0.2", "0.3", "0.4", "0.6", "0.8", "1.2", "1.5"):
    cols.append([f"=T.INV.2T({p},{d})" for d in range(3, 13)])
case(
    "val-t-inv-joint-abs-rel-stop",
    "Student-t ordinary inverse: the AS 109 iterate is kept when the last step meets BOTH 10^(-13-2.5/a^2-0.5/p^2) "
    "and |step| <= EPSILON*x. A dense df sweep would show a df window if one existed. Column A T.INV.2T(.5,df) "
    "for df 2..30 (rows 1-29) and 123456785..123456795 (rows 30-40); B:H T.INV.2T(p,df) p=.2,.3,.4,.6,.8,1.2,1.5, "
    "rows df 3..12.",
    cols,
)

# ---------------------------------------------------------------- G12 q ~ .25
pp = ["0.24", "0.245", "0.249", "0.2499", "0.25-4*2^-54", "0.25-2^-54", "0.25", "0.25+2^-54", "0.25+4*2^-54",
      "0.2501", "0.251", "0.255", "0.26", "0.75", "0.125", "0.375"]
cols = [[f"=T.INV({p},{d})" for p in pp] for d in (5, 7, 8, 9, 6, 10, 1000000)]
case(
    "val-t-inv-quarter-stop",
    "The unresolved ordinary stop near q=.25 (AS 109/R19 iterates reproduce df 5,7,8,9 and large df but not all "
    "controls). Columns df 5,7,8,9,6,10,1E6; rows p = .24,.245,.249,.2499,.25-4ulp,.25-1ulp,.25,.25+1ulp,.25+4ulp,"
    ".2501,.251,.255,.26,.75,.125,.375 (ulp = 2^-54 below .25, 2^-54 used both sides).",
    cols,
)

# ---------------------------------------------------------------- G13 tiny centre region
cs = ["2^-52", "1E-15", "1E-14", "1E-12", "1E-10", "1E-8", "1E-6"]
cols = [[f"=TINV(1-{c},{d})" for c in cs] for d in range(1, 19)]
cols.append([f"=T.INV.2T(1-{c},15)" for c in cs])
cols.append([f"=T.INV.2T(1-{c},16)" for c in cs])
cols.append([f"=CONFIDENCE.T(1-{c},1,16)" for c in cs])
case(
    "val-t-inv-tiny-centre-region",
    "Tiny two-tailed centre: the engine reuses the small-beta inverse only in bpser's split-prefactor branch "
    "(min(a,b)<1, max(a,b)<8, i.e. df<16) and where |a(1-b)z/(a+1)| <= EPSILON. Columns A:R TINV(1-c,df) df=1..18; "
    "S,T T.INV.2T df 15,16; U CONFIDENCE.T(1-c,1,16) (df 15). Rows c = 2^-52,1E-15,1E-14,1E-12,1E-10,1E-8,1E-6.",
    cols,
)

# ---------------------------------------------------------------- G14 tiny tails
pp = ["1E-20", "1E-50", "1E-100", "1E-150", "1E-200", "1E-250", "1E-300"]
cols = [[f"=T.INV({p},{d})" for p in pp] for d in (1, 2, 3, 5, 8, 15, 30)]
cols.append([f"=T.INV(1E-{k},2)" for k in range(100, 210, 10)])
case(
    "val-t-inv-small-beta-tails",
    "Student-t inverse deep tails: grouped log seed + Newton on the beta CDF, and at df 2 the switch to the log seed "
    "once the squared Newton correction underflows (between 1E-100 and 1E-200). Columns A:G df 1,2,3,5,8,15,30 at "
    "p=1E-20..1E-300; H T.INV(1E-k,2) k=100..200 step 10.",
    cols,
)

# ---------------------------------------------------------------- G15 T.DIST df1 / large-df centre
ts = ["1E-10", "0.001", "0.1", "0.3", "0.5", "0.9", "1-2^-53", "1", "1+2^-52", "1.005-1", "1.5"]
cols = [[f"=T.DIST.2T({t},1)" for t in ts], [f"=T.DIST.RT({t},1)" for t in ts]]
for t in ("0.1", "0.5", "0.9", "0.999999", "1", "1.000001"):
    cols.append([f"=T.DIST.2T({t},{d})" for d in (500, 998, 999, 1000, 1001, 10000, 1E6, 1E8, 1E10)])
case(
    "val-t-dist-df1-and-large-df-centre",
    "T-2930: at df 1, 0<t<=1, the tail is evaluated at rounded y=df/(df+t^2); for t<1 and df>=1000 the stable "
    "central fraction. Columns A,B T.DIST.2T / T.DIST.RT at df 1 (rows t=1E-10..1.5); C:H T.DIST.2T(t,df) for "
    "t=.1,.5,.9,.999999,1,1.000001, rows df=500,998,999,1000,1001,1E4,1E6,1E8,1E10.",
    cols,
)

# ---------------------------------------------------------------- G16 NORM.S.INV central fingerprint
xs = [0.700 + 0.0005 * i for i in range(31)] + [0.84 + 0.001 * i for i in range(21)] + \
     [0.49 + 0.001 * i for i in range(21)]
cols = [[f"=NORM.S.INV(0.5+{lit(round(x, 4))}/2)" for x in xs],
        [f"=NORM.S.INV(0.5-{lit(round(x, 4))}/2)" for x in xs],
        [f"=NORM.S.INV(0.5+{k}*2^-30)" for k in range(1, 41)],
        [f"=NORM.S.INV(0.5+{k}*2^-12)" for k in range(1, 41)]]
case(
    "val-norm-s-inv-central-fingerprint",
    "Data for discriminating inverse-erf candidates: central NORM.S.INV across x=|2(p-.5)| in .700..715 (the "
    "suspected split near sqrt(.5)), .84..86 (end of the divide-image band) and .49..51, both signs; and dyadic "
    "low-x steps p=.5+k*2^-30, .5+k*2^-12 (k=1..40) whose x is exact. Columns A (p>.5), B (p<.5), C, D.",
    cols,
)

# ---------------------------------------------------------------- G17 LOGNORM.DIST density holdout
cols = []
for s in (0.1, 0.3, 0.7, 1.3, 2.9):
    for m in (-2, 0, 1, 3.5):
        cols.append([f"=LOGNORM.DIST({lit(math.exp(m + k * s))},{m},{s},FALSE)" for k in (-3, -1.5, -0.5, 0, 0.5, 1.5, 3)])
cols.append([f"=LOGNORM.DIST({x},1,0.5,FALSE)" for x in ("1", "2", "4", "8", "0.5", "0.25", "40", "2.718281828459045")])
case(
    "val-lognorm-density-holdout",
    "Holdout for T-4321's density: EXP(-h)*(1+e) from the four-term two_sum chain ((d*d)/(s*s))/2, LN(s), "
    "LN(2.5066282746310007), LN(x), positive low-part sign. Non-dyadic sigmas the 120-cell grid lacked. Columns: "
    "sigma in .1,.3,.7,1.3,2.9 (outer) x mean in -2,0,1,3.5 (inner); rows x = EXP(mean+k*sigma) as literals, "
    "k=-3,-1.5,-.5,0,.5,1.5,3. Last column dyadic controls at (x,1,.5).",
    cols,
)

# ---------------------------------------------------------------- G18 Loader store chain
rows_b, rows_r, rows_n = [], [], []
for n in (5, 10, 20, 30, 50, 100):
    for p in (0.1, 0.2, 0.3, 0.4, 0.6, 0.7, 0.8, 0.9):
        for k in range(1, n):
            for kk, mm in ((k, n * p), (n - k, n * (1 - p))):
                if mm > 0 and abs(abs(kk - mm) - 0.2 * (kk + mm)) < 1e-9:
                    rows_b.append(f"=BINOM.DIST({k},{n},{p},FALSE)")
                    rows_r.append(f"=BINOM.DIST.RANGE({n},{p},{k})")
                    rows_n.append(f"=NEGBINOM.DIST({n - k},{k},{p},FALSE)")
                    break
tails = ["=BINOM.DIST.RANGE(7,1E-20,6)", "=BINOM.DIST(6,7,1E-20,FALSE)", "=BINOM.DIST(3,1000,1E-20,FALSE)",
         "=BINOM.DIST(2,10,1E-100,FALSE)", "=BINOM.DIST(5,1000000,1E-12,FALSE)", "=BINOM.DIST(50,100,0.999,FALSE)",
         "=BINOM.DIST(1,1000000000,1E-12,FALSE)", "=BINOM.DIST(999,1000,0.01,FALSE)", "=BINOM.DIST(0,1000,0.5,FALSE)",
         "=BINOM.DIST(2,10,0.3,FALSE)", "=NEGBINOM.DIST(6,1,1E-20,FALSE)", "=NEGBINOM.DIST(50,50,0.999,FALSE)"]
case(
    "val-binom-loader-store-chain",
    "Loader PMF chains (T-2146): bd0's x87 cutoff |k-M| < RN64(.2*(k+M)) at exact-boundary integer pairs, and the "
    "stored log-branch chain of the unscaled binomial vs the extended scaled NEGBINOM path. Column A BINOM.DIST(k,n,p,"
    "FALSE), B BINOM.DIST.RANGE one-point, C NEGBINOM.DIST(n-k,k,p,FALSE), for every (n,p,k) with n in "
    "{5,10,20,30,50,100}, p in {.1..9} where |k-np| or |(n-k)-n(1-p)| equals .2*(sum) in exact arithmetic; D far-tail "
    "log-branch controls.",
    [rows_b, rows_r, rows_n, tails],
)

# ---------------------------------------------------------------- G19 moments x87
# Data sets live in cells (row i, columns F onward) so exact doubles such as 1+2^-52 survive (a formula literal or
# array constant keeps only 15 significant digits and cannot hold an expression).
SETS = [[f"{k}E-15", "1", "2"] for k in range(1, 10)]
SETS += [["0.1", "42", big, "0.1"] for big in ("123456789.123", "987654321.5", "1E9+0.5", "33554432.1")]
SETS += [["0.2", "7", "3.14159", "-2.5"], ["1", "0.1", "100", "100", "0.5", "0.3", "0.3", "1E-15", "0.5"],
         ["1", "1", "1", "1", "1+2^-52"], ["1E15", "1E15+1", "1E15+2", "1E15+4"], ["1", "2", "3", "4", "1000000"],
         ["-1E-300", "0", "1E-300", "2E-300"], ["1"] * 9 + ["2"], ["0.1", "0.2", "0.3", "0.4", "0.5", "10"],
         ["1E8", "1E8+1", "1E8+3", "1E8+7", "1E8+15"], ["3"] * 16 + ["7"],
         ["1.5", "1.5", "-1.5", "-1.5", "0.1"], ["1", "1.5", "2", "2.5", "100"], ["2^-30", "2^-29", "3*2^-30", "1"]]
cols: list[list[str]] = [[], [], []]
data_cols: dict[int, list[str]] = {}
for i, st in enumerate(SETS):
    r = i + 1
    cols[0].append(f"=SKEW(F{r}:Z{r})")
    cols[1].append(f"=SKEW.P(F{r}:Z{r})")
    cols[2].append(f"=KURT(F{r}:Z{r})")
    for j, v in enumerate(st):
        data_cols.setdefault(j, [""] * len(SETS))[i] = "=" + v
cols += [[], []]  # D, E empty
cols += [data_cols[j] for j in sorted(data_cols)]
case(
    "val-moments-x87-adversarial",
    "SKEW/SKEW.P/KURT via two-pass mean, z=(x-mean)/sqrt(m2/divisor), stored ops rounded to a 64-bit significand "
    "then binary64, cubes (z*z)*z, fourth powers (z*z)*(z*z). Adversarial sets where double rounding and the power "
    "order are visible. Row i: A SKEW, B SKEW.P, C KURT of the data set in F{i}:Z{i} (data cells are formulas so "
    "exact doubles survive).",
    cols,
)

# ---------------------------------------------------------------- G20 IRR
flows = ["{0.3,-2.5,1.005}", "{0.3,-2.5,1.0049}", "{0.3,-2.5,1.0051}", "{-0.3,2.5,-1.005}", "{3,-25,10.05}",
         "{0.03,-0.25,0.1005}", "{0.3,-2.6,1.005}", "{0.3,-2.4,1.005}", "{1,7,-1}", "{1,20,-1}", "{1,6,-1}",
         "{1,8,-1}", "{42,1,1,-1}", "{40,1,1,-1}", "{44,1,1,-1}", "{-400,1,1,1,1,1}", "{-300,1,1,1,1,1}",
         "{-500,1,1,1,1,1}", "{-4,4}", "{-1,0,0,0,0,0,0,0,0,0,0,0,1E-6}", "{-1E8,1,1,1,1,1,1,1,1E8}"]
cols = [[f"=IRR({f})" for f in flows], [f"=IRR({f},0.5)" for f in flows], [f"=IRR({f},-0.5)" for f in flows]]
case(
    "val-irr-negative-discount-and-caps",
    "IRR secant search: may an intermediate iterate cross d=r/(1+r)>1 (negative discount) and still converge; final "
    "discount must be positive; stalled-step residual scaled by discounted terms. Columns default guess, .5, -.5; "
    "rows are the flow vectors listed in PROBES.md (variants around IRR({.3,-2.5,1.005}), {1,7,-1}, the two open "
    "fallback targets and a long-run case).",
    cols,
    tags=["M9", "financial", "irr", "probe"],
)

# ---------------------------------------------------------------- G21 BETA.INV number vs #NUM! at tiny shapes
TINY = ["1E-3", "1E-6", "1E-9", "1E-12", "1E-14", "1E-15", "1E-16", "1E-20"]
PT = ["0.001", "0.1", "0.5", "0.9", "0.999"]
cols = []
for other in ("0.5", "1", "5"):
    for t in TINY:
        cols.append([f"=BETA.INV({p},{other},{t})" for p in PT] + [f"=BETA.INV({p},{t},{other})" for p in PT])
cols.append(["=BETA.INV(0.120463618823915,4.85116218981846,3.81506961561941E-15)",
             "=BETA.INV(0.540825862774324,2.11840621275937E-15,0.719272803854574)",
             "=BETA.INV(2.27966476300884E-261,0.000693561996566429,2.69166894976264E-279)",
             "=BETA.INV(0.1,5,1E-14)", "=BETAINV(0.1,5,1E-14)", "=BETA.DIST(0.9999999999999999,5,1E-14,TRUE)",
             "=BETAINV(0.120463618823915,4.85116218981846,3.81506961561941E-15)",
             "=BETAINV(0.540825862774324,2.11840621275937E-15,0.719272803854574)"])
both = []
for a in ("1E-3", "7E-4", "1E-6"):
    for b in ("1E-100", "1E-279"):
        for p in ("1E-100", "2.3E-261", "1E-300", "0.5"):
            both.append(f"=BETA.INV({p},{a},{b})")
cols.append(both)
case(
    "val-beta-inv-tiny-shape-num-boundary",
    "Where BETA.INV switches from a number to #NUM! at tiny shapes and extreme p (unbracketed or stalled AS 109 "
    "searches: the engine returns finite values where Excel gave #NUM! at accuracy-table s1 A89, A94, A184). "
    "Columns A:X: other shape .5 (A:H), 1 (I:P), 5 (Q:X) x tiny shape 1E-3,1E-6,1E-9,1E-12,1E-14,1E-15,1E-16,1E-20; "
    "rows 1-5 BETA.INV(p,other,tiny), rows 6-10 BETA.INV(p,tiny,other), p=.001,.1,.5,.9,.999. Column Y: the audit "
    "cells, BETA.INV(.1,5,1E-14), BETAINV twins and the forward check BETA.DIST(1-2^-53,5,1E-14). Column Z: both "
    "shapes tiny, (a,b) in {1E-3,7E-4,1E-6}x{1E-100,1E-279}, p=1E-100,2.3E-261,1E-300,.5.",
    cols,
)

# ---------------------------------------------------------------- G22 BETA.INV at DBL_MIN probabilities
PD = ["1E-300", "1E-307", "4E-308", "3E-308", "2.5E-308", "2.22507385850721E-308", "2^-1022",
      "2.2250738585072E-308", "2^-1022*(1+2^-52)", "2^-1022*(1+2^-40)", "2^-1021"]
SH = [("16.0270968311551", "5.57477023741975"), ("16", "5.5"), ("2", "2"), ("5", "5"), ("1", "1"), ("0.5", "0.5"),
      ("3", "30"), ("30", "3"), ("100", "100"), ("1.5", "300")]
cols = [[f"=BETA.INV({p},{a},{b})" for p in PD] for a, b in SH]
cols.append([f"=BETA.DIST(BETA.INV({p},16.0270968311551,5.57477023741975),16.0270968311551,5.57477023741975,TRUE)"
             for p in PD])
cols.append([f"=F.INV({p},5,5)" for p in PD])
case(
    "val-beta-inv-dbl-min-residual",
    "BETA.INV at probabilities around DBL_MIN, where the first CDF residual is subnormal: does Excel flush it "
    "(accepting the start) or keep it? Accuracy-table s1 A111 BETA.INV(2.22507385850721E-308,16.03,5.57) is "
    "3.6855664841262965E-20 in Excel. Columns A:J shapes (16.0270968311551,5.57477023741975),(16,5.5),(2,2),(5,5),"
    "(1,1),(.5,.5),(3,30),(30,3),(100,100),(1.5,300); K the forward CDF of column A's quantile; L F.INV(p,5,5). Rows "
    "p=1E-300,1E-307,4E-308,3E-308,2.5E-308,2.22507385850721E-308,2^-1022,2.2250738585072E-308 (subnormal literal),"
    "2^-1022*(1+2^-52), 2^-1022*(1+2^-40), 2^-1021 (literals below DBL_MIN are not parsed as numbers, so none are used).",
    cols,
)

# ---------------------------------------------------------------- G23 BETA.INV round trip at large shapes
rows = []
for tot in (20, 100, 200, 500, 2000, 10000, 100000):
    for frac in (0.01, 0.05, 0.3, 0.5, 0.7, 0.95):
        a = round(tot * frac, 3) or 0.2
        b = round(tot - a, 3)
        m = a / (a + b)
        sd = math.sqrt(a * b / ((a + b) ** 2 * (a + b + 1)))
        for k in (-2, 0, 2):
            x = m + k * sd
            if 0 < x < 1:
                rows.append((lit(x), lit(a), lit(b)))
rows += [("0.0122366954923074", "241.098", "19333"), ("0.0319328911260104", "1063", "32055.693")]
cols = [[f"={x}" for x, _, _ in rows], [f"={a}" for _, a, _ in rows], [f"={b}" for _, _, b in rows]]
cols.append([f"=BETA.DIST(A{i + 1},B{i + 1},C{i + 1},TRUE)" for i in range(len(rows))])
cols.append([f"=BETA.INV(D{i + 1},B{i + 1},C{i + 1})" for i in range(len(rows))])
cols.append([f"=E{i + 1}-A{i + 1}" for i in range(len(rows))])
cols.append([f"=BETAINV(D{i + 1},B{i + 1},C{i + 1})" for i in range(len(rows))])
case(
    "val-beta-inv-large-shape-round-trip",
    "BETA.INV(BETA.DIST(x,a,b,TRUE),a,b) at large shapes: where Excel returns x exactly (crossing) vs an early "
    "AS 109 iterate (t3443-basym-sweep A101/A102 return x; the engine returns an iterate 1-2E-15 away). Rows: total "
    "shape 20..1E5 x a/(a+b) .01..95 x x at mean and mean +-2 sd, then the two audit cells. Columns A x, B a, C b, "
    "D forward CDF, E BETA.INV(D), F E-A, G BETAINV(D).",
    cols,
)

# ---------------------------------------------------------------- G24 T.DIST central tail at df 999-1001
TT = [f"{0.05 * i:.2f}" for i in range(1, 20)] + ["0.8513", "1-2^-53", "1E-8"]
cols = []
for d in (999, 1000, 1001):
    cols.append([f"=T.DIST.RT({t},{d})" for t in TT])
    cols.append([f"=T.DIST.2T({t},{d})" for t in TT])
    cols.append([f"=T.DIST({t},{d},TRUE)" for t in TT])
case(
    "val-t-dist-df1000-central-endpoint",
    "Is the finite-series domain inclusive at df 1000 for 0<t<1 (the stable central fraction should start at df "
    "1001, not 1000; m9-t-dist-rt A123 and t0969 B1553 are Excel controls at df 1000)? Columns per df 999,1000,1001: "
    "T.DIST.RT, T.DIST.2T, T.DIST(TRUE); rows t=.05..95 step .05, .8513, 1-2^-53, 1E-8.",
    cols,
)


if __name__ == "__main__":
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    for cid, c in CASES.items():
        (out / f"{cid}.case.json").write_text(json.dumps(c, indent=1) + "\n")
        print(cid, len(c["sheets"][0]["cells"]))
