import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,4);print(T)
for _ in range(T):
    n=r.randint(2,4);A=[r.randint(1,3) for _ in range(n)]
    while sum(A)>8:A[r.randrange(n)]=1
    cand=[(l1,i1,l2,i2) for l1 in range(1,n+1) for l2 in range(l1+1,n+1) for i1 in range(1,A[l1-1]+1) for i2 in range(1,A[l2-1]+1)]
    P=r.sample(cand,r.randint(0,min(len(cand),n+1)));print(n,len(P));print(*A)
    for p in P:print(*p)
