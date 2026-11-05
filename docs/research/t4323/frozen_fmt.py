import json,math,sys
def r15(v): return float('%.15g'%v) if v else v
def model(x,s):
    S=86400*s; y=x+0.5/S; fl=math.floor(y); fr=y-fl
    F=math.floor(r15(fr*86400*s)); el=math.floor(r15(y*86400))
    return F%s, F//s, el
def round_half(x,s):
    U=math.floor(x*86400*s+0.5); return U%s,(U//s)%86400,U//s
def fmt(f,s,p):
    frac,clock,el=p; dig=len(str(s))-1; fr=f"{frac:0{dig}d}"
    return {"[h]:mm.0":f"{el//3600}:{el//60%60:02d}.{fr}",
     "h:mm:ss.0":f"{clock//3600%24}:{clock//60%60:02d}:{clock%60:02d}.{fr}",
     "h:mm:ss.000":f"{clock//3600%24}:{clock//60%60:02d}:{clock%60:02d}.{fr}",
     "[s].0":f"{el}.{fr}","s.00":f"{clock%60}.{fr}",
     "[h]:mm:ss.000":f"{el//3600}:{el//60%60:02d}:{el%60:02d}.{fr}",
     "[m]:ss.00":f"{el//60}:{el%60:02d}.{fr}",
     "mm:ss.0":f"{clock//60%60:02d}:{clock%60:02d}.{fr}"}[f]
