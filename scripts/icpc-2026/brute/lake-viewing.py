import sys
from collections import deque
# Literal search over (waypoint, arcs walked). Paths are free moves; arcs at most once.
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    n,m=int(next(it)),int(next(it));A=[int(next(it)) for _ in range(n)]
    arcs=[((l,w),(l,(w+1)%A[l])) for l in range(n) for w in range(A[l])]
    nb={}
    for _ in range(m):
        l1,i1,l2,i2=(int(next(it)) for _ in range(4))
        a,b=(l1-1,i1-1),(l2-1,i2-1);nb.setdefault(a,[]).append(b);nb.setdefault(b,[]).append(a)
    full=(1<<len(arcs))-1;start=((0,0),0);seen={start};dq=deque([start]);ok=False
    while dq:
        pos,used=dq.popleft()
        if used==full:ok=True;break
        nxt=[(q,used) for q in nb.get(pos,[])]
        for k,(u,v) in enumerate(arcs):
            if used>>k&1:continue
            if u==pos:nxt.append((v,used|1<<k))
            if v==pos:nxt.append((u,used|1<<k))
        for s in nxt:
            if s not in seen:seen.add(s);dq.append(s)
    print("YES" if ok else "NO")
