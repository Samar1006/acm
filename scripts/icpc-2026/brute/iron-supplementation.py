import sys
from fractions import Fraction as F
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    m,d,p=int(next(it)),int(next(it)),int(next(it));x=F(0)
    for _ in range(d):x=(x+m)*F(100-p,100)
    print("%.12f"%float(x))
