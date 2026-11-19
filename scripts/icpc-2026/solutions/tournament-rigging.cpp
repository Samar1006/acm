#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
int N;vector<ll>b;vector<string>s;
// dp over the bracket: cost for each player to emerge from a block of players [lo, hi).
vector<ll> solve(int lo,int hi){if(hi-lo==1)return{0};int mid=(lo+hi)/2;auto L=solve(lo,mid),R=solve(mid,hi);vector<ll>res(hi-lo,LLONG_MAX);
 for(int x=lo;x<mid;x++){ll best=LLONG_MAX;for(int y=mid;y<hi;y++){ll c=R[y-mid]+(s[x][y]=='1'?0:b[y]);best=min(best,c);}res[x-lo]=L[x-lo]+best;}
 for(int y=mid;y<hi;y++){ll best=LLONG_MAX;for(int x=lo;x<mid;x++){ll c=L[x-lo]+(s[y][x]=='1'?0:b[x]);best=min(best,c);}res[y-lo]=R[y-mid]+best;}
 return res;}
int main(){int T;scanf("%d",&T);while(T--){int k;scanf("%d",&k);N=1<<k;b.assign(N,0);s.assign(N,"");static char buf[5000];for(int i=0;i<N;i++){scanf("%lld %s",&b[i],buf);s[i]=buf;}printf("%lld\n",solve(0,N)[0]);}}
