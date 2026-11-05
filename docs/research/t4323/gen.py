import json,sys,hashlib
FMTS=[("[h]:mm.0",10),("h:mm:ss.0",10),("[s].0",10),("s.00",100),("[h]:mm:ss.000",1000)]
DAYS=[0,1,45000]
cells={};meta={}
r=1
for f,s in FMTS:
  for d in DAYS:
    for j in range(10):
      k=(j+0.5)/s  # seconds
      ks=repr(k).rstrip('0') if False else ('%g'%k)
      dpart=f"{d}+" if d else ""
      fe=f.replace('"','""')
      cells[f"A{r}"]=f'=TEXT({dpart}{ks}/86400,"{f}")'
      meta[f"A{r}"]=(f,s,d,k)
      r+=1
case={"format":"rustsheet-case/v1","description":"T-4323: TEXT of a serial whose seconds fraction sits on a rounding tie ((j+0.5) units of the shown fraction past whole day d) under [h]:mm.0, h:mm:ss.0, [s].0, s.00 and [h]:mm:ss.000, to separate Excel's fraction rounding models.","tags":["M10","text","probe"],"sheets":[{"name":"Sheet1","cells":cells}],"expected":{}}
json.dump(case,open(sys.argv[1],'w'),indent=2);open(sys.argv[1],'a').write('\n')
json.dump(meta,open('meta1.json','w'))
print(r-1)
