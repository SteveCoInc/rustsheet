import json,sys,random,math
random.seed(4323)
FMTS=[("[h]:mm.0",10),("h:mm:ss.0",10),("[s].0",10),("s.00",100),("[h]:mm:ss.000",1000),("[m]:ss.00",100),("mm:ss.0",10),("h:mm:ss.000",1000)]
cells={};meta={};r=1
for f,s in FMTS:
  for i in range(12):
    d=random.choice([0,0,1,2,7,30,365,1000,20000,45000,60000,2958000])
    # log-uniform seconds offset in [1e-3, 86399] , half of them snapped to a tie of the shown unit
    sec=10**random.uniform(-3,math.log10(86399))
    if i%2==0: sec=(math.floor(sec*s)+0.5)/s
    sec=float('%.6g'%sec)
    dpart=f"{d}+" if d else ""
    cells[f"A{r}"]=f'=TEXT({dpart}{sec:.6g}/86400,"{f}")'
    meta[f"A{r}"]=(f,s,d,float(f"{sec:.6g}"))
    r+=1
case={"format":"rustsheet-case/v1","description":"T-4323 fresh sweep (generated after the fraction rounding model was frozen): TEXT of whole days plus log-uniform seconds offsets, half on a tie of the shown fraction, under elapsed and clock formats with .0, .00 and .000.","tags":["M10","text","probe"],"sheets":[{"name":"Sheet1","cells":cells}],"expected":{}}
s=json.dumps(case,indent=2)+"\n";open(sys.argv[1],'w').write(s)
json.dump(meta,open('meta2.json','w'));print(r-1)
