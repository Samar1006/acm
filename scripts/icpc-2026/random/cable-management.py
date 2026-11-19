import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,4);print(T)
for _ in range(T):
    V=r.randint(1,6);W=r.randint(1,6);cells=[(x,y) for x in range(1,V+1) for y in range(1,W+1)]
    if len(cells)<2:cells=[(1,1),(1,2)]
    P=r.sample(cells,r.randint(2,min(len(cells),14)));print(len(P))
    for p in P:print(*p)
