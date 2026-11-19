import sys
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    n,k=int(next(it)),int(next(it));res=set()
    for mask in range(1<<k):
        x=n
        for i in range(k):x=-(-x//2) if mask>>i&1 else x//2
        res.add(x)
    print(len(res))
