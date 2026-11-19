import sys
it=iter(sys.stdin.read().split());T=int(next(it))
for _ in range(T):
    s=next(it);t=next(it);p=set(zip(s,t))
    print("YES" if len(p)==len(set(s))==len(set(t)) else "NO")
