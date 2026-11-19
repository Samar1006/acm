import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,3);print(T)
for _ in range(T):
    n=r.randint(1,7);s="".join(r.choice("WM" if r.random()<0.7 else "WWM") for _ in range(n));ts=sorted(r.sample(range(1,30),n))
    print(n);print(s);print(*ts);q=r.randint(1,6);print(q)
    for _ in range(q):
        l=r.randint(1,n);print(l,r.randint(l,n))
