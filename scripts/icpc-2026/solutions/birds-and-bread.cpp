#include <bits/stdc++.h>
using namespace std;
typedef long long ll;typedef unsigned long long ull;
// For a pair, |r_i - r_j| = gap of throw times stays fixed and r_i + r_j = 2T - t_i - t_j grows.
// Distance D < gap: never meet. D == gap: tangent (1) once both exist. D > gap: 0, then 1 at
// equality (only if D is an integer of the right parity), then 2 forever.
ull isqrt(ull x){ull r=sqrtl((long double)x);while(r*r>x)r--;while((r+1)*(r+1)<=x)r++;return r;}
int main(){int T;scanf("%d",&T);while(T--){int n,q;scanf("%d %d",&n,&q);vector<ll>t(n),x(n),y(n);for(int i=0;i<n;i++)scanf("%lld %lld %lld",&t[i],&x[i],&y[i]);
 vector<pair<ll,int>>ev;ev.reserve((size_t)n*n);
 for(int i=0;i<n;i++)for(int j=i+1;j<n;j++){ull dx=llabs(x[i]-x[j]),dy=llabs(y[i]-y[j]);ull D2=dx*dx+dy*dy;ll gap=llabs(t[i]-t[j]);ll S=t[i]+t[j];ll tmax=max(t[i],t[j]);
  ull r=isqrt(D2);bool sq=r*r==D2;
  if((ull)gap*(ull)gap>D2)continue;
  if((ull)gap*(ull)gap==D2){ev.push_back({tmax,1});continue;}
  // smallest k = 2T - S with k > D, k with the parity of S
  if(sq&&((ll)(r%2)==(S&1))){ll Te=((ll)r+S)/2;ev.push_back({Te,1});ev.push_back({Te+1,1});}
  else{ull k=r+1;if((ll)(k%2)!=(S&1))k++;ll Tt=((ll)k+S)/2;ev.push_back({Tt,2});}}
 sort(ev.begin(),ev.end());size_t p=0;ll cur=0;
 for(int i=0;i<q;i++){ll Q;scanf("%lld",&Q);while(p<ev.size()&&ev[p].first<=Q)cur+=ev[p++].second;printf("%lld\n",cur);}}}
