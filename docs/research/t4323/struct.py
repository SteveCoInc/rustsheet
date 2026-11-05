import json,math
from eps import load
data=load([('meta1.json','t4323-fraction-ties'),('meta2.json','t4323-fraction-sweep'),('meta3.json','t4323-fraction-sweep-2')])
def fl(v): return math.floor(v)
O={
 'O1':lambda x,s:(lambda y:(y-fl(y))*86400*s)(x+0.5/(86400*s)),
 'O1b':lambda x,s:(lambda y:(y-fl(y))*(86400*s))(x+0.5/(86400*s)),
 'O2':lambda x,s:(lambda t2:(t2-fl(t2/86400)*86400)*s)(x*86400+0.5/s),
 'O3':lambda x,s:((x-fl(x))+0.5/(86400*s))*86400*s,
 'O4':lambda x,s:((x-fl(x))*86400+0.5/s)*s,
 'O5':lambda x,s:(x-fl(x))*86400*s+0.5,
 'O5b':lambda x,s:(x-fl(x))*(86400*s)+0.5,
 'O6':lambda x,s:(x*86400-fl(x)*86400)*s+0.5,
 'O7':lambda x,s:(x*86400*s - fl(x)*86400*s)+0.5,
 'O8':lambda x,s:((x*86400)%86400)*s+0.5,
 'O9':lambda x,s:((x*86400*s)%(86400*s))+0.5,
 'O10':lambda x,s:((x-fl(x))*24*60*60)*s+0.5,
 'O11':lambda x,s:(((x-fl(x))*24*60*60*s)+0.5),
 'O12':lambda x,s:((((x-fl(x))*24)*60)*60+0.5/s)*s,
}
def frac_digit(e,s):
    dig=len(str(s))-1; return e.rsplit('.',1)[1][:dig] if '.' in e else None
for nm,fn in O.items():
    ok=0;miss=[]
    for (o,c,f,ss,d,k,e) in data:
        x=(d+k/86400) if d else k/86400
        P=fn(x,ss); got=f"{fl(P)%ss:0{len(str(ss))-1}d}"
        if got==frac_digit(e,ss): ok+=1
        else: miss.append((o[-7:],c,f,d,k,e,got))
    print(nm,ok,'/',len(data),miss[:3])
