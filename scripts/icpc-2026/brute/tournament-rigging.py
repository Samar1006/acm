import sys,itertools
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    k=int(next(it));N=1<<k;B=[];S=[]
    for _ in range(N):B.append(int(next(it)));S.append(next(it))
    best=None
    # every match: 0 = fair, 1 = bribe the first player, 2 = bribe the second
    for choice in itertools.product(range(3),repeat=N-1):
        players=list(range(N));cost=0;ci=0
        while len(players)>1:
            nxt=[]
            for a,b in zip(players[::2],players[1::2]):
                c=choice[ci];ci+=1
                if c==0:w=a if S[a][b]=='1' else b
                elif c==1:w=b;cost+=B[a]
                else:w=a;cost+=B[b]
                nxt.append(w)
            players=nxt
        if players[0]==0 and (best is None or cost<best):best=cost
    print(best)
