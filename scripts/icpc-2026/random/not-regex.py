import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,4);print(T)
for _ in range(T):
    al="abcdefghijklmnopqrstuvwxyz"[:r.choice([1,2,3,26])];n=r.randint(1,30)
    if r.random()<0.4:
        unit="".join(r.choice(al) for _ in range(r.randint(1,3)));s=(unit*n)[:n]
    else:s="".join(r.choice(al) for _ in range(n))
    m=r.randint(1,min(n,6));print(s);print("".join(r.choice("abcdefghijklmnopqrstuvwxyz"[:r.choice([3,26])]) for _ in range(m)))
