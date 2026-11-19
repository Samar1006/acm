import sys
# Optimal policy over sets of completed tasks, solved exactly as a fixed point:
# for a guess x of E(empty), E(S) = min_i (t + a E(S+i) + g x) / (1 - b); find x = E_x(empty).
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    n=int(next(it));tasks=[tuple(int(next(it)) for _ in range(4)) for _ in range(n)]
    if any(s==0 for _,s,_,_ in tasks):print("impossible");continue
    full=(1<<n)-1
    def E0(x):
        E=[0.0]*(1<<n)
        for S in range(full-1,-1,-1):
            best=float('inf')
            for i,(t,s,f,c) in enumerate(tasks):
                if S>>i&1:continue
                tot=s+f+c;a,bb,g=s/tot,f/tot,c/tot
                best=min(best,(t+a*E[S|1<<i]+g*x)/(1-bb))
            E[S]=best
        return E[0]
    lo,hi=0.0,1.0
    while E0(hi)>hi:hi*=2
    for _ in range(200):
        mid=(lo+hi)/2
        if E0(mid)>mid:lo=mid
        else:hi=mid
    print("%.12e"%lo)
