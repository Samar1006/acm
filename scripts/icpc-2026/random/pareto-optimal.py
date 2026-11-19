import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,6);print(T)
for _ in range(T):
    n=r.randint(1,9);V=r.randint(1,8);pts=set()
    while len(pts)<n and len(pts)<V*V:pts.add((r.randint(1,V),r.randint(1,V)))
    pts=list(pts);r.shuffle(pts);print(len(pts),r.randint(1,V))
    for p in pts:print(*p)
