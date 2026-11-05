import mpmath as mp, math, json
mp.mp.prec=300
def r64(v):
    with mp.workprec(64): return +mp.mpf(v)
def r53(v):
    return float(mp.mpf(v))
L2E=r64(1/mp.log(2))
def xexp(a):
    """Excel EXP V6 of a (mpf allowed: extended register), returns extended w*2^n unrounded-to-double (mpf, 64-bit sig)"""
    a=mp.mpf(a)
    if a==0: return mp.mpf(1)
    t=r64(a*L2E); n=mp.nint(t); f=t-n
    u=r64(mp.power(2,abs(f))-1); w=r64(1+u)
    if f<0: w=r64(1/w)
    return mp.ldexp(w,int(n))
K=0.3989422804014327
SQ=2.5066282746310007
SQc=math.sqrt(2*math.pi)
cells=json.load(open('cells.json'))
