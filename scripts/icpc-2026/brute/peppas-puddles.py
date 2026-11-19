import sys,itertools
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    n=int(next(it));L=next(it);R=next(it);best=0
    for order in itertools.permutations(range(n)):
        splashed=[False]*n;jumped=[False]*n;cnt=0
        for p in order:
            if splashed[p]:continue
            jumped[p]=True;cnt+=1
            if p>0 and L[p-1]=='1' and not jumped[p-1]:splashed[p-1]=True
            if p<n-1 and R[p]=='1' and not jumped[p+1]:splashed[p+1]=True
        best=max(best,cnt)
    print(best)
