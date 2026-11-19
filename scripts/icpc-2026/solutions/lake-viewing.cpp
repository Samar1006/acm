#include <bits/stdc++.h>
using namespace std;
// Every waypoint has even degree from the lake arcs, and paths may be walked any number of
// times, so walking a spanning tree of paths twice gives an Euler circuit: the answer is YES
// exactly when all lakes are connected by paths.
int f[200005];int fd(int x){while(f[x]!=x)x=f[x]=f[f[x]];return x;}
int main(){int T;scanf("%d",&T);while(T--){int n,m;scanf("%d %d",&n,&m);for(int i=1;i<=n;i++){long long a;scanf("%lld",&a);f[i]=i;}int comp=n;for(int i=0;i<m;i++){int l1,l2;long long i1,i2;scanf("%d %lld %d %lld",&l1,&i1,&l2,&i2);int a=fd(l1),b=fd(l2);if(a!=b){f[a]=b;comp--;}}puts(comp==1?"YES":"NO");}}
