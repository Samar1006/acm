import sys
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    n=int(next(it));P=[(int(next(it)),int(next(it))) for _ in range(n)]
    lines=[('x',v) for v in sorted({x for x,_ in P})]+[('y',v) for v in sorted({y for _,y in P})]
    best=None
    for mask in range(1,1<<len(lines)):
        ch=[lines[i] for i in range(len(lines)) if mask>>i&1]
        if best is not None and len(ch)>=best:continue
        if not all(any((o=='x' and v==x) or (o=='y' and v==y) for o,v in ch) for x,y in P):continue
        # lines connect iff both orientations are used, or there is just one line
        kinds={o for o,_ in ch}
        if len(ch)==1 or len(kinds)==2:best=len(ch)
    print(best)
