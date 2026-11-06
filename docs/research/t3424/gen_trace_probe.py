import json, math, sys
sys.path.insert(0, sys.argv[1])
from model_trace import R
CELLS = [
 ("sweep","A272",(4,-70299017520.6175,254784242189.789,-1503672.27062336,0)),
 ("sweep","A296",(351,-1684985188.67608,607019725001.509,-15796156082.0366,0)),
 ("fresh","A59",(2,6278360237.88483,-12528846789.2511,-27774543.9929057,0)),
 ("fresh","A70",(35,12273893066.0886,-427389409614.586,-2130919589.40718,0)),
 ("fresh","A74",(321,-2890484105.96678,934408648396.347,-10935378183.8393,1)),
 ("fresh","A77",(329,-42389848.4869782,13945255025.8794,-23095230.0549541,0)),
 ("fresh","A89",(14,-240193754.37537,4611780278.38432,-1249534255.10969,0)),
 ("fresh","A90",(136,1438873775.69802,-196213656238.621,580155950.046744,0)),
 ("fresh","A95",(2,-2432513870.20006,-3318506.12039095,4868366502.37616,1)),
 ("fresh","A157",(1,25550828180.0966,-25201858262.4494,-325879402.767792,0)),
 ("fresh","A162",(1,35007935575.2514,-35005912619.7461,-2026687.67999046,1)),
 ("fresh","A173",(15,-9926778189.99403,138904429552.499,-209302706.994644,0)),
 ("fresh","A251",(1,101776720612.892,-676573751810.73,583281411526.675,1)),
 ("fresh","A272",(1,-203158570592.69,3667932.84612167,206903872457.554,1)),
 ("fresh","A275",(2,478550742813.386,-925555108604.956,277863719.42142,1)),
 ("fresh","A292",(6,-101229151545.993,550032163728.64,-13767204.6882258,0)),
]
def iterates(a, guess=0.1):
    f=lambda r: R(r,a); out=[]
    r0,f0=guess,f(guess); r1=r0-math.copysign(0.001,f0); f1=f(r1); out+= [r0,r1]
    for step in range(1,201):
        if f1==f0: break
        from rustsheet_tools.fitlab.exact import X87
        r2=X87.add(r1,-X87.div(X87.mul(f1,X87.add(r1,-r0)),X87.add(f1,-f0)))
        if not math.isfinite(r2): break
        f2=f(r2); out.append(r2)
        if abs(r2-r1)<1e-7 and abs(f2)<1e-7: break
        r0,f0,r1,f1=r1,f1,r2,f2
    seen=[]; [seen.append(x) for x in out if x not in seen]; return seen
from rustsheet_tools.fitlab.exact import X87
def cut15(x):
    from decimal import Decimal, ROUND_DOWN
    if x==0: return 0.0
    d=Decimal(repr(x)); e=d.adjusted()
    return float(d.scaleb(-e).quantize(Decimal('1.'+'0'*14),rounding=ROUND_DOWN).scaleb(e))
def lit(x):
    x=float(x)
    if cut15(x)==x and len(repr(abs(x)).replace('.','').replace('-','').lstrip('0').split('e')[0])<=15: return repr(x)
    a=float('%.14e'%x); b=x-a
    b15=cut15(b)
    assert X87.add(a,b15)==x, (x,a,b,b15)
    return "(%s+%s)"%(('%.14e'%x), ('%.14e'%b))
def args_s(a): n,pmt,pv,fv,t=a; return f"{lit(n) if n!=int(n) else int(n)},{pmt!r},{pv!r},{fv!r},{t}"
cells={}; meta=[]; row=1
for case,cell,a in CELLS:
    n,pmt,pv,fv,t=a
    its=iterates(a)
    # keep the first 6 and the last 24 iterates (the noise floor is at the end)
    pick = its if len(its)<=30 else its[:6]+its[-24:]
    for k,r in enumerate(pick):
        cells[f"A{row}"]=f"=FV({lit(r)},{n},{pmt!r},{pv!r},{t})"; meta.append((f"A{row}",case,cell,"FV",r)); row+=1
    if n<=2:
        for r in its[-12:]:
            cells[f"A{row}"]=f"=RATE({args_s(a)},{lit(r)})"; meta.append((f"A{row}",case,cell,"RATE",r)); row+=1
print(row-1, file=sys.stderr)
json.dump({"cells":cells,"meta":meta}, open(sys.argv[2],"w"), indent=0)
