import json,math,sys
from frozen_fmt import fmt
EPS=2.0**-52
SN={
 'mul1eps':lambda v: v*(1+EPS),
 'addveps':lambda v: v+v*EPS,
 'exact_rel_n':lambda v: float(math.floor(v)+1) if (math.floor(v)+1-v)<=(math.floor(v)+1)*EPS else v,
 'r15':lambda v: float('%.15g'%v) if v else v,
}
def model(x,s,sn):
    S=86400*s; y=x+0.5/S; fl=math.floor(y); fr=y-fl
    F=math.floor(sn(fr*86400*s)); return F%s,F//s,math.floor(sn(y*86400))
def load(pairs):
    data=[]
    for mf,of in pairs:
        meta=json.load(open(mf)); ora=json.load(open(f'../../../eval/suites/probes/cases/{of}.oracle.json'))['sheets']['Sheet1']
        for c,(f,s,d,k) in meta.items(): data.append((of,c,f,s,d,k,ora[c]['v']))
    return data
if __name__=='__main__':
    pairs=[tuple(a.split(':')) for a in sys.argv[1:]]
    data=load(pairs)
    for nm,sn in SN.items():
        miss=[]
        for (o,c,f,s,d,k,e) in data:
            x=(d+k/86400) if d else k/86400
            p=fmt(f,s,model(x,s,sn))
            if p!=e: miss.append((o,c,f,d,k,e,p))
        print(nm,len(data)-len(miss),'/',len(data),miss[:4])
