import random,sys
r=random.Random(int(sys.argv[1]));T=r.randint(1,3);print(T)
trip=[(3,4,5),(5,12,13),(8,15,17),(7,24,25),(20,21,29)]
for _ in range(T):
    n=r.randint(2,6);B=[];seen=set();base=r.randint(0,10**9-10**6)
    for i in range(n):
        while True:
            a,b,c=r.choice(trip);s=r.randint(1,30000000)
            x=r.choice([0,a*s,-a*s,10**9,-10**9]);y=r.choice([0,b*s,-b*s,10**9,-10**9])
            t=base+r.randint(0,10**6)
            if (t,x,y) not in seen:seen.add((t,x,y));B.append((t,x,y));break
    B.sort();
    cand=set()
    for i in range(n):
        for j in range(i+1,n):
            D=((B[i][1]-B[j][1])**2+(B[i][2]-B[j][2])**2)**0.5;S=B[i][0]+B[j][0]
            for d in (-2,-1,0,1,2):
                v=int((D+S)/2)+d
                if 0<=v<=10**9:cand.add(v)
    cand.update(r.randint(base,10**9) for _ in range(5))
    qs=sorted(cand)[:40];print(n,len(qs))
    for b in B:print(*b)
    for q in qs:print(q)
