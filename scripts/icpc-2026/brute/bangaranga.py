import sys,math
it=iter(sys.stdin.read().split());T=int(next(it));o=[]
for _ in range(T):
    d=int(next(it));q=int(next(it))
    for _ in range(q):
        op=next(it);k=int(next(it));e=10*round(math.log10(k))
        d=d+e if op=="BANGA" else d-e;o.append(str(d))
print("\n".join(o))
