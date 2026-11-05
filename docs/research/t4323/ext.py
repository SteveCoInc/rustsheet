import json,math,sys
from fractions import Fraction as Fr
def rnd(q,bits):
    q=Fr(q)
    if q==0: return q
    sgn=1 if q>0 else -1; a=abs(q)
    e=a.numerator.bit_length()-a.denominator.bit_length()
    if Fr(2)**e>a: e-=1
    if Fr(2)**(e+1)<=a: e+=1
    scale=Fr(2)**(bits-1-e); m=a*scale
    n=math.floor(m); r=m-n
    if r>Fr(1,2) or (r==Fr(1,2) and n%2): n+=1
    return sgn*n/scale
D=lambda q:rnd(q,53); X=lambda q:rnd(q,64)
def cand(name,x,s):
    S=86400*s; x=Fr(x)
    if name=='X1':
        y=X(x+X(Fr(1,2)/S)); fr=y-math.floor(y); P=X(X(fr*86400)*s); el=math.floor(X(y*86400))
    elif name=='X1d':  # products stored to double
        y=X(x+X(Fr(1,2)/S)); fr=y-math.floor(y); P=D(X(X(fr*86400)*s)); el=math.floor(D(X(y*86400)))
    elif name=='Xy_d':  # y stored to double, then ext
        y=D(X(x+X(Fr(1,2)/S))); fr=y-math.floor(y); P=X(X(fr*86400)*s); el=math.floor(X(y*86400))
    elif name=='Xs':  # scale first S then multiply
        y=X(x+X(Fr(1,2)/S)); fr=y-math.floor(y); P=X(fr*S); el=math.floor(X(y*86400))
    elif name=='D':
        y=D(x+D(Fr(1,2)/S)); fr=y-math.floor(y); P=D(D(fr*86400)*s); el=math.floor(D(y*86400))
    F=math.floor(P); return F%s,F//s,el
sys.argv=[sys.argv[0]]
