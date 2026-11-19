import sys
def score(x):
    tot=0
    for j,c in enumerate(x):tot+=(ord(c)-96)*(1+sum(1 for k in range(j) if x[k]<c))
    return tot
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    s=next(it);t=next(it);n,m=len(s),len(t);best=None
    for i in range(n-m+1):
        cur=list(s);p=s[i:i+m];j=i
        while True:
            cur[j:j+m]=list(t)
            nj=next((k for k in range(j+m,n-m+1) if "".join(cur[k:k+m])==p),None)
            if nj is None:break
            j=nj
        sc=score(cur);best=sc if best is None else min(best,sc)
    print(best)
