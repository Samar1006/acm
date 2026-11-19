import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,4);print(T)
for _ in range(T):
    n=r.randint(2,11);mode=r.random();print(n)
    print(*[(i-1 if mode<0.2 else r.randint(max(1,i-2),i-1) if mode<0.5 else r.randint(1,i-1)) for i in range(2,n+1)])
