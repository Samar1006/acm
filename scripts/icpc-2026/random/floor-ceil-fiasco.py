import random,sys
r=random.Random(int(sys.argv[1]));small=len(sys.argv)>2;T=r.randint(1,50);print(T)
for _ in range(T):
    n=r.choice([r.randint(1,10**9),r.randint(1,100),2**r.randint(0,29),2**r.randint(1,29)-1,2**r.randint(1,29)+1,10**9])
    print(n,r.randint(0,14 if small else 30))
