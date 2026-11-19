import sys
it=iter(sys.stdin.read().split());T=int(next(it));o=[]
for _ in range(T):
    n,q=int(next(it)),int(next(it));B=[tuple(int(next(it)) for _ in range(3)) for _ in range(n)]
    for _ in range(q):
        Q=int(next(it));tot=0
        for i in range(n):
            for j in range(i+1,n):
                (ti,xi,yi),(tj,xj,yj)=B[i],B[j]
                if Q<ti or Q<tj:continue
                ri,rj=Q-ti,Q-tj;D2=(xi-xj)**2+(yi-yj)**2
                if D2==0:continue
                lo,hi=(ri-rj)**2,(ri+rj)**2
                tot+=2 if lo<D2<hi else (1 if D2 in (lo,hi) else 0)
        o.append(str(tot))
print("\n".join(o))
