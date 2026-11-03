import sys,subprocess,os,hashlib,struct,collections
cells=[l.rstrip('\n').split('\t') for l in open(sys.argv[1])]
def ho(c): 
    d=hashlib.sha256(f"rustsheet-fitlab-holdout/v1\0{c[0]}\0{c[1]}".encode()).digest(); return int.from_bytes(d[:8],'big')%1000<300
def run(v):
    p=subprocess.run([sys.argv[2]],input="\n".join(c[2] for c in cells)+"\n",capture_output=True,text=True,env={**os.environ,"RS_V":str(v)},check=True)
    return p.stdout.split("\n")
def norm(x): return 'ERR' if x.startswith('E:') else repr(float(x))
for v in sys.argv[3:]:
    o=run(v); s=collections.Counter()
    for c,g in zip(cells,o):
        side='test' if ho(c) else 'train'; s[side+'_n']+=1; s[side]+= norm(c[3])==norm(g)
    print(v, f"train {s['train']}/{s['train_n']} ({s['train']/s['train_n']:.3f})  test {s['test']}/{s['test_n']} ({s['test']/s['test_n']:.3f})")
