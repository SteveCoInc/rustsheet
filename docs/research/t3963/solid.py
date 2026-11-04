import sys,subprocess,os,struct,collections
cells=[l.rstrip('\n').split('\t') for l in open(sys.argv[1])]
def run(v):
    p=subprocess.run([sys.argv[2]],input="\n".join(c[2] for c in cells)+"\n",capture_output=True,text=True,env={**os.environ,"RS_V":str(v)},check=True)
    return p.stdout.split("\n")
def b(x): return struct.unpack('<q',struct.pack('<d',x))[0]
def cls(e,g):
    if e.startswith('E:') and g.startswith('E:'): return 'exact'
    if e.startswith('E:') or g.startswith('E:'): return 'solid'
    d=abs(b(float(e))-b(float(g)))
    return 'exact' if d==0 else ('ulp' if d<=4 else 'solid')
outs={v:run(v) for v in sys.argv[3:]}
for v,o in outs.items():
    c=collections.Counter(cls(x[3],y) for x,y in zip(cells,o))
    print(v,dict(c))
