import sys
from fractions import Fraction as F
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    n=int(next(it));pts=[(0,0)]+[(int(next(it)),int(next(it))) for _ in range(n)]
    # time as a function of distance is piecewise linear; find it at 5280
    for (d0,t0),(d1,t1) in zip(pts,pts[1:]):
        if d0<=5280<=d1:
            print("%.9f"%float(t0+F(t1-t0)*(5280-d0)/(d1-d0)));break
