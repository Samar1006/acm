import sys
it=iter(sys.stdin.read().split());T=int(next(it));o=[]
for _ in range(T):
    n=int(next(it));s=next(it);t=[int(next(it)) for _ in range(n)];q=int(next(it))
    for _ in range(q):
        l,r=int(next(it))-1,int(next(it))-1;span=t[r]-t[l];best=None;bestd=None
        for d in range(1,span+2):
            until=None;cnt=0;ok=True
            for i in range(l,r+1):
                vis=until is not None and t[i]<until
                if s[i]=='M':
                    if vis:ok=False;break
                else:
                    if not vis:cnt+=1
                    until=t[i]+d
            if not ok:continue
            if best is None or cnt<best or (cnt==best):
                if best is None or cnt<best:best=cnt
                bestd=d if cnt==best else bestd
        # d beyond the span behaves like span+1, so reaching it means d is unbounded
        o.append(f"{best} {-1 if bestd==span+1 else bestd}")
print("\n".join(o))
