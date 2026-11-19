import sys
from fractions import Fraction as F
P=998244353
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    n=int(next(it));par=[0,0]+[int(next(it)) for _ in range(n-1)];dep=[0]*(n+1)
    for i in range(2,n+1):dep[i]=dep[par[i]]+1
    E=F(0)
    for mask in range(1<<(n-1)):
        purple=[False]*(n+1);purple[1]=True;pr=F(1);ok=True
        for i in range(2,n+1):
            want=bool(mask>>(i-2)&1)
            if purple[par[i]]:pr*=F(1,2);purple[i]=want
            elif want:ok=False;break
        if not ok:continue
        E+=pr*len({dep[i] for i in range(1,n+1) if purple[i]})
    print(E.numerator%P*pow(E.denominator,P-2,P)%P)
