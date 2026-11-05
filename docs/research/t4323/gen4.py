import json,sys,random,math
from eps import SN,model
from frozen_fmt import fmt
random.seed(43233)
FMTS=[("[h]:mm.0",10),("h:mm:ss.0",10),("[s].0",10),("s.00",100),("[h]:mm:ss.000",1000),("[m]:ss.00",100),("mm:ss.0",10),("h:mm:ss.000",1000)]
cells={};meta={};r=1;sep=0;tries=0
while r<=120 and tries<2000000:
    tries+=1
    f,s=random.choice(FMTS)
    d=random.choice([0,1,2,3,5,7,30,365,1000,20000,45000,60000,2958000])
    sec=10**random.uniform(-3,math.log10(86399))
    sec=(math.floor(sec*s)+0.5)/s
    sec=float('%.7g'%sec)
    x=(d+sec/86400) if d else sec/86400
    outs={nm:fmt(f,s,model(x,s,sn)) for nm,sn in SN.items()}
    plain=fmt(f,s,model(x,s,lambda v:v))
    differ=len(set(outs.values())|{plain})>1
    pass
    sep+=differ
    dpart=f"{d}+" if d else ""
    cells[f"A{r}"]=f'=TEXT({dpart}{sec:.7g}/86400,"{f}")'; meta[f"A{r}"]=(f,s,d,float(f"{sec:.7g}")); r+=1
case={"format":"rustsheet-case/v1","description":"T-4323 fresh sweep 3 (generated after the half-step register model was frozen): rounding ties of the shown seconds fraction past whole days, under elapsed and clock formats with .0, .00 and .000.","tags":["M10","text","probe"],"sheets":[{"name":"Sheet1","cells":cells}],"expected":{}}
open(sys.argv[1],'w').write(json.dumps(case,indent=2)+"\n"); json.dump(meta,open('meta4.json','w')); print(r-1,sep,tries)
