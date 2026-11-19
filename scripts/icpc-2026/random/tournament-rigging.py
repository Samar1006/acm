import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,3);print(T)
for _ in range(T):
    k=r.randint(1,3);N=1<<k;print(k);W=[[0]*N for _ in range(N)]
    for i in range(N):
        for j in range(i+1,N):
            if r.random()<0.5:W[i][j]=1
            else:W[j][i]=1
    for i in range(N):print(r.randint(1,9),"".join(map(str,W[i])))
