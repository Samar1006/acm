import sys,itertools
it=iter(sys.stdin.read().split());T=int(next(it))
def comps(R,k):
    if k==0:
        if R==0:yield ()
        return
    for x in range(R+1):
        for rest in comps(R-x,k-1):yield (x,)+rest
for _ in range(T):
    n,p=int(next(it)),int(next(it));J=[];V=[]
    for _ in range(n):J.append(int(next(it)));V.append(int(next(it)))
    ans=-1
    for c in range(n+1):
        R=p-sum(V[:c]);winners=set()
        for comp in comps(R,n-c):
            tot=[J[i]+V[i] for i in range(c)]+[J[c+i]+comp[i] for i in range(n-c)]
            m=max(tot);w=[i for i in range(n) if tot[i]==m]
            winners.add(w[0] if len(w)==1 else None)
        if len(winners)==1 and None not in winners:ans=c;break
    print(ans)
