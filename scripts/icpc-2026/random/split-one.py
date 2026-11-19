import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,50);print(T)
for _ in range(T):
    n=r.randint(1,50)
    while True:
        ds=sorted(r.sample(range(1,10001),n));ts=sorted(r.sample(range(1,10001),n))
        if ds[-1]>=5280:break
    if r.random()<0.2 and 5280 not in ds and n>1: ds[r.randrange(n-1)]=None
    ds=[d for d in ds if d is not None];ts=ts[:len(ds)]
    if r.random()<0.3:
        k=r.randrange(len(ds))
        if 5280 not in ds and (k==0 or ds[k-1]<5280) and (k==len(ds)-1 or ds[k+1]>5280): ds[k]=5280
    print(len(ds))
    for d,t in zip(ds,ts):print(d,t)
