import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,50);print(T)
for _ in range(T):print(r.randint(1,50),r.randint(1,50),r.choice([1,99,50,r.randint(1,99)]))
