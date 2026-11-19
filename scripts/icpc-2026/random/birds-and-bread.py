import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,3);print(T)
for _ in range(T):
    n=r.randint(1,7);V=r.choice([2,4,8]);ts=sorted(r.randint(0,10) for _ in range(n));seen=set();B=[]
    for t in ts:
        while True:
            p=(t,r.randint(-V,V),r.randint(-V,V))
            if p not in seen:seen.add(p);B.append(p);break
    qs=sorted(r.sample(range(0,30),r.randint(1,8)));print(n,len(qs))
    for b in B:print(*b)
    for x in qs:print(x)
