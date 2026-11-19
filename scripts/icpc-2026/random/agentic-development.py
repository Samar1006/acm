import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,4);print(T)
for _ in range(T):
    n=r.randint(1,5);print(n)
    for _ in range(n):
        while True:
            s,f,c=[r.choice([0,r.randint(0,10)]) for _ in range(3)]
            if s+f+c>0:break
        if r.random()<0.9 and s==0:s=r.randint(1,10)
        print(r.randint(1,20),s,f,c)
