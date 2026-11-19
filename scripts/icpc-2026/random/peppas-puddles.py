import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,5);print(T)
for _ in range(T):
    n=r.randint(2,7);q=r.random();b=lambda:"".join("1" if r.random()<q else "0" for _ in range(n-1))
    print(n);print(b());print(b())
