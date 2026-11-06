"""T-4419: the 15-digit rounding done in doubles (r*1e15 and other scalings) on several remainders
(python3 search_double15.py, from this directory). Scored on probes/t4419-band-ulp-map; best 869/913."""
import json,math
m=json.load(open('meta1.json'))
o=json.load(open('../../../eval/suites/probes/cases/t4419-band-ulp-map.oracle.json'))['sheets']['Sheet1']
cells=[]
for c,(f,s,k,n,yh,p) in m.items():
    tie=round((k+0.0005)*1000)
    cells.append((float.fromhex(yh),tie,o[c]['v']==f"0.{tie:03d}",c))
H=0.5/86400000
def cut(v):
    v-=math.floor(v)
    for f in (24,60,60): v*=f; v-=math.floor(v)
    return v
srcs={'cut':lambda y:cut(y+H),'cut+off':lambda y:cut(y+H+2**-72),'*86400':lambda y:(y+H)*86400,
      'ms(24,60,60,1000)/1000':lambda y:((y+H)*24*60*60*1000)/1000, 'y*86400+.0005':lambda y:y*86400+0.0005,
      'y*24+h,*3600':lambda y:(y*24+0.5/3600000)*3600}
def mul15(r):
    e=math.floor(math.log10(r)); p=14-e
    return {'r*1e15':r*10.0**p,'r*10^p seq':(lambda v:[v:=v*10 for _ in range(p)][-1])(r),
            'r*1e3*1e12':r*1e3*10.0**(p-3),'r/1e-p':r/10.0**-p, 'r*1e16/10': r*10.0**(p+1)/10}
res=[]
for sn,sf in srcs.items():
    for mn in ['r*1e15','r*10^p seq','r*1e3*1e12','r/1e-p','r*1e16/10']:
      for rmode in ('half_up','half_even'):
        good=0
        for y,tie,up,c in cells:
            r=sf(y); q=mul15(r)[mn]
            iq=math.floor(q+0.5) if rmode=='half_up' else round(q)
            shown=iq//10**12
            if (shown>=tie)==up: good+=1
        res.append((good,sn,mn,rmode))
for r in sorted(res,reverse=True)[:12]: print(r)
