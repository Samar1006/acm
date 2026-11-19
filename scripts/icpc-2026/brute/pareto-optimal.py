import sys
it=iter(sys.stdin.read().split());T=int(next(it));o=[]
for _ in range(T):
    n,k=int(next(it)),int(next(it));P=[(int(next(it)),int(next(it))) for _ in range(n)]
    ok=[i+1 for i,(mi,ci) in enumerate(P) if not any(j!=i and ((mj>=mi+k and cj>=ci) or (mj>=mi and cj>=ci+k)) for j,(mj,cj) in enumerate(P))]
    o.append(str(len(ok)));o.append(" ".join(map(str,ok)))
print("\n".join(o))
