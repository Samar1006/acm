import random,sys,string
r=random.Random(int(sys.argv[1]));big=len(sys.argv)>2
T=r.randint(1,50 if big else 6);print(T)
for _ in range(T):
    n=r.randint(1,50 if big else 8);al=string.ascii_lowercase[:r.randint(1,26)]
    s="".join(r.choice(al) for _ in range(n))
    while True:
        if r.random()<0.5:
            perm=list(string.ascii_lowercase);r.shuffle(perm);m=dict(zip(string.ascii_lowercase,perm))
            t="".join(m[c] for c in s)
            if r.random()<0.3:
                i=r.randrange(n);t=t[:i]+r.choice(string.ascii_lowercase)+t[i+1:]
        else:t="".join(r.choice(al) for _ in range(n))
        if t!=s:break
    print(s);print(t)
