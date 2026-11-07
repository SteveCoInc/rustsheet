import mpmath as mp, math
mp.mp.dps=40
def temme(A,X):
    lam=X/A
    if abs(lam-1)<mp.mpf('1e-25'):
        return mp.mpf(0.5),mp.mpf(0.5)
    e=mp.sign(lam-1)*mp.sqrt(2*(lam-1-mp.log(lam)))
    c=1/(lam-1)-1/e
    w=mp.exp(-A*e*e/2)/mp.sqrt(2*mp.pi*A)
    Q=mp.erfc(e*mp.sqrt(A/2))/2+w*c
    P=mp.erfc(-e*mp.sqrt(A/2))/2-w*c
    return P,Q
def dens(A,X): return mp.exp((A-1)*mp.log(X)-X-mp.loggamma(A))
def newton(a,x0,p,tail,cap=2000,tol=1e-15,mode='raw'):
    A=mp.mpf(a);x=mp.mpf(x0);p=mp.mpf(p)
    for i in range(cap):
        P,Q=temme(A,x)
        f=dens(A,x)
        if f==0: return i,'f0',x
        if tail=='L': s=(P-p)/f
        else: s=-(Q-p)/f
        x1=x-s
        if x1<=0: return i,'neg',x
        if abs(s)<=tol*x: return i+1,'ok',x1
        x=x1
    return cap,'cap',x
def odds_newton(a,x0,p,tail,cap=600,tol=1e-15):
    A=mp.mpf(a);x=mp.mpf(x0);p=mp.mpf(p)
    for i in range(cap):
        P,Q=temme(A,x)
        f=dens(A,x)
        if f==0: return i,'f0',x
        if tail=='L':
            r=p/(1-p); s=(P*Q-r*Q*Q)/f
        else:
            r=(1-p)/p; s=(P*Q-r*Q*Q)/f
        x1=x-s
        if x1<=0: return i,'neg',x
        if abs(s)<=tol*x: return i+1,'ok',x1
        x=x1
    return cap,'cap',x
