import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,4);print(T)
for _ in range(T):
    n=r.randint(2,4);p=r.randint(1,5);J=sorted(r.randint(0,6) for _ in range(n))
    cuts=sorted(r.randint(0,p) for _ in range(n-1));V=[b-a for a,b in zip([0]+cuts,cuts+[p])];r.shuffle(V)
    print(n,p)
    for a,b in zip(J,V):print(a,b)
