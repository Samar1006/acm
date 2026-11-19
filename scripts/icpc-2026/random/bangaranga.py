import random,sys
r=random.Random(int(sys.argv[1]));big=len(sys.argv)>2
T=r.randint(1,50 if big else 5);print(T)
for _ in range(T):
    q=r.randint(1,50 if big else 6);print(r.randint(1,200),q)
    for _ in range(q):print(r.choice(["BANGA","RANGA"]),10**r.randint(1,9))
