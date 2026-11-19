#include <bits/stdc++.h>
using namespace std;
typedef long long ll;const ll P=998244353,HALF=(P+1)/2;
ll pw(ll b,ll e){ll r=1;b%=P;while(e){if(e&1)r=r*b%P;b=b*b%P;e>>=1;}return r;}
// g_u[k] = P(no purple node k levels below u | u purple) = prod over children (1 + g_v[k-1]) / 2.
// Long-path decomposition: inherit the deepest child's array (stored reversed) under a lazy
// affine map value = a*raw + b, and multiply in the shorter children element by element.
struct Arr{vector<ll>raw;ll a=1,ai=1,b=0;};
int main(){int T;scanf("%d",&T);while(T--){int n;scanf("%d",&n);vector<int>par(n+1,0),h(n+1,0),heavy(n+1,0);vector<vector<int>>ch(n+1);
 for(int i=2;i<=n;i++){scanf("%d",&par[i]);ch[par[i]].push_back(i);}
 for(int i=n;i>=2;i--){int p=par[i];if(h[i]+1>h[p]||heavy[p]==0){if(h[i]+1>=h[p]){h[p]=max(h[p],h[i]+1);}}}
 for(int u=1;u<=n;u++){int best=0;for(int v:ch[u])if(!best||h[v]>h[best])best=v;heavy[u]=best;}
 vector<Arr>A(n+1);
 for(int u=n;u>=1;u--){Arr cur;
  if(heavy[u]){cur=std::move(A[heavy[u]]);
   // apply x -> (1+x)/2
   cur.a=cur.a*HALF%P;cur.ai=cur.ai*2%P;cur.b=(1+cur.b)%P*HALF%P;}
  // depth 0: u itself is purple, so the value is 0
  cur.raw.push_back((0-cur.b+P)%P*cur.ai%P);
  int L=cur.raw.size();
  for(int v:ch[u]){if(v==heavy[u])continue;Arr&w=A[v];int lv=w.raw.size();
   for(int k=0;k<lv;k++){ll gv=(w.a*w.raw[lv-1-k]+w.b)%P;ll f=(1+gv)%P*HALF%P;ll&r=cur.raw[L-1-(k+1)];ll val=(cur.a*r+cur.b)%P;val=val*f%P;r=(val-cur.b+P)%P*cur.ai%P;}
   vector<ll>().swap(w.raw);}
  A[u]=std::move(cur);}
 Arr&R=A[1];ll ans=0;for(ll r:R.raw){ll g=(R.a*r+R.b)%P;ans=(ans+1-g+P)%P;}printf("%lld\n",ans);}}
