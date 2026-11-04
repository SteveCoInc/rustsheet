import sys,subprocess,os,collections
cells=[l.rstrip('\n').split('\t') for l in open(sys.argv[1])]
bin_=sys.argv[2]
def run(v):
    p=subprocess.run([bin_],input="\n".join(c[2] for c in cells)+"\n",capture_output=True,text=True,env={**os.environ,"RS_V":str(v)},check=True)
    return p.stdout.split("\n")[:len(cells)]
def norm(x):
    if x.startswith('E:'): return 'ERR' if 'NUM' in x else x
    return repr(float(x))
res={}
for v in map(int,sys.argv[3:]): res[v]=run(v)
base=sys.argv[3]
for v,out in res.items():
    ok=collections.Counter(); tot=collections.Counter()
    for c,o in zip(cells,out):
        e=norm(c[3]); g=norm(o)
        tot[c[0]]+=1; ok[c[0]]+= (e==g)
    print(f"V{v}: {sum(ok.values())}/{sum(tot.values())}")
    if len(sys.argv)>4:
        for k in sorted(tot): print(f"   {k:45s} {ok[k]}/{tot[k]}")
# diffs vs first variant
b=res[int(base)]
for v,out in res.items():
    if v==int(base): continue
    gain=loss=0
    for c,o,o0 in zip(cells,out,b):
        e=norm(c[3]); g=norm(o); g0=norm(o0)
        gain+= (e==g and e!=g0); loss+=(e!=g and e==g0)
    print(f"V{v} vs V{base}: +{gain} -{loss}")
