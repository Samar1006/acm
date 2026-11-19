#include <bits/stdc++.h>
using namespace std;
typedef long long ll;const ll INF=LLONG_MAX;
// Searching wizards only decrease as d grows, so the best d is the largest feasible one:
// D = min over Muggles after the first wizard in [l, r] of (Muggle time - previous wizard time).
// Then the first wizard searches, plus every later wizard whose gap to the previous one is >= D.
int main(){int T;scanf("%d",&T);static char buf[300005];while(T--){int n;scanf("%d",&n);scanf("%s",buf);vector<ll>t(n);for(int i=0;i<n;i++)scanf("%lld",&t[i]);
 vector<int>nextW(n+1,n);for(int i=n-1;i>=0;i--)nextW[i]=buf[i]=='W'?i:nextW[i+1];
 vector<ll>G(n,INF),gw(n,-1);int pw=-1;for(int i=0;i<n;i++){if(buf[i]=='M'){if(pw>=0)G[i]=t[i]-t[pw];}else{if(pw>=0)gw[i]=t[i]-t[pw];pw=i;}}
 int LG=1;while((1<<LG)<=n)LG++;vector<vector<ll>>sp(LG,vector<ll>(n));sp[0]=G;for(int k=1;k<LG;k++)for(int i=0;i+(1<<k)<=n;i++)sp[k][i]=min(sp[k-1][i],sp[k-1][i+(1<<(k-1))]);
 auto rmq=[&](int a,int b){if(a>b)return INF;int k=31-__builtin_clz(b-a+1);return min(sp[k][a],sp[k][b-(1<<k)+1]);};
 int q;scanf("%d",&q);vector<int>qa(q),qb(q);vector<ll>qd(q),cntv(q,0);vector<int>fwv(q);
 for(int i=0;i<q;i++){int l,r;scanf("%d %d",&l,&r);l--;r--;int fw=nextW[l];fwv[i]=fw;qb[i]=r;if(fw>r){qd[i]=-2;continue;}qd[i]=rmq(fw+1,r);qa[i]=fw+1;}
 // offline: count wizards j in [qa, qb] with gw[j] >= D, processing D in decreasing order
 vector<int>wz;for(int i=0;i<n;i++)if(gw[i]>=0)wz.push_back(i);sort(wz.begin(),wz.end(),[&](int x,int y){return gw[x]>gw[y];});
 vector<int>ord;for(int i=0;i<q;i++)if(qd[i]>=0&&qd[i]!=INF)ord.push_back(i);sort(ord.begin(),ord.end(),[&](int x,int y){return qd[x]>qd[y];});
 vector<int>bit(n+1,0);auto add=[&](int i){for(i++;i<=n;i+=i&-i)bit[i]++;};auto sum=[&](int i){int s=0;for(i++;i>0;i-=i&-i)s+=bit[i];return s;};
 size_t p=0;for(int qi:ord){while(p<wz.size()&&gw[wz[p]]>=qd[qi])add(wz[p++]);cntv[qi]=sum(qb[qi])-(qa[qi]>0?sum(qa[qi]-1):0);}
 for(int i=0;i<q;i++){if(qd[i]==-2)printf("0 -1\n");else if(qd[i]==INF)printf("1 -1\n");else printf("%lld %lld\n",1+cntv[i],qd[i]);}}}
