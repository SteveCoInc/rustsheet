import sys, collections; sys.path.insert(0, sys.argv[1])
from score_rate import *
sys.path.insert(0, str(ROOT/"docs/research/t3424"))
from model_trace import run
C=collections.Counter(); fam=collections.defaultdict(collections.Counter)
meta=json.load(open(sys.argv[1]+"/probe_meta.json"))
ox=json.load(open(str(ROOT)+"/eval/suites/probes/cases/t3424-rate-noise-trace.oracle.json"))["sheets"]["Sheet1"]
src=open(ROOT/"docs/research/t3424/gen_trace_probe.py").read()
CELLS=eval(src[src.index("CELLS = [")+8:src.index("]\ndef iterates")+1]); A={(c,x):a for c,x,a in CELLS}
for cell,case,c,kind,r in meta:
    if kind!="RATE": continue
    res=run(A[(case,c)],r,True,False)
    e=ox[cell]; enum=e["t"]!="n"
    key=(res[0], "excel #NUM!" if enum else ("excel = model" if res[0] in("conv","hold") and e["v"]==res[1] else "excel other number"))
    C[key]+=1; fam[c][key]+=1
for k,v in sorted(C.items()): print(k,v)
for c,v in fam.items(): print(c,dict(v))
print("--- by hold point")
G=collections.defaultdict(collections.Counter)
for cell,case,c,kind,r in meta:
    if kind!="RATE": continue
    res=run(A[(case,c)],r,True,False)
    if res[0]!="hold": continue
    e=ox[cell]; enum=e["t"]!="n"
    f=R(res[1],A[(case,c)])
    G[(c,res[1],"%.3g"%f)]["NUM" if enum else ("same" if e["v"]==res[1] else "other")]+=1
for k,v in sorted(G.items()): print(k,dict(v))
