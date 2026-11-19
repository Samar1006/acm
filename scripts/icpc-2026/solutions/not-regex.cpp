#include <bits/stdc++.h>
using namespace std;
typedef long long ll;typedef unsigned long long ull;
// score = sum v(s_j) + sum over pairs k<j with s_k<s_j of v(s_j). Replacing a chain of r equal
// blocks p by t changes: pairs between each block and everything else (computed with original
// characters elsewhere), pairs inside blocks, and a correction for block-block pairs.
const ull MOD=(1ULL<<61)-1;ull mulm(ull a,ull b){__uint128_t c=(__uint128_t)a*b;ull lo=(ull)(c&MOD),hi=(ull)(c>>61);ull r=lo+hi;if(r>=MOD)r-=MOD;return r;}
int main(){int T;scanf("%d",&T);static char sb[200005],tb[200005];ull BASE=1000003ULL+(ull)chrono::steady_clock::now().time_since_epoch().count()%1000000007ULL;
 while(T--){scanf("%s %s",sb,tb);string s=sb,t=tb;int n=s.size(),m=t.size();vector<int>a(n),b(m);for(int i=0;i<n;i++)a[i]=s[i]-'a';for(int i=0;i<m;i++)b[i]=t[i]-'a';
  auto v=[](int c){return (ll)c+1;};
  vector<array<int,26>>pre(n+1);pre[0].fill(0);for(int i=0;i<n;i++){pre[i+1]=pre[i];pre[i+1][a[i]]++;}
  // L(j) = v(s_j) * #{k<j: s_k<s_j}; R(j) = sum over k>j with s_j<s_k of v(s_k)
  vector<ll>LR(n+1,0),Lp(n+1,0);ll S0=0;
  {array<ll,26>sufv;sufv.fill(0);vector<ll>R(n);for(int j=n-1;j>=0;j--){ll r=0;for(int c=a[j]+1;c<26;c++)r+=sufv[c];R[j]=r;sufv[a[j]]+=v(a[j]);}
   for(int j=0;j<n;j++){ll less=0;for(int c=0;c<a[j];c++)less+=pre[j][c];ll L=v(a[j])*less;S0+=v(a[j])*(1+less);LR[j+1]=LR[j]+L+R[j];Lp[j+1]=Lp[j]+L;}}
  // letter-count helpers: K(x,y) = sum_c cx[c] * sum_{d>c} v(d) cy[d]
  auto K=[&](const array<ll,26>&cx,const array<ll,26>&cy){ll acc=0,res=0;for(int c=25;c>=0;c--){res+=cx[c]*acc;acc+=v(c)*cy[c];}return res;};
  array<ll,26>ct;ct.fill(0);for(int x:b)ct[x]++;
  ll It=0;{array<ll,26>seen;seen.fill(0);for(int x:b){for(int c=0;c<x;c++)It+=seen[c]*v(x);seen[x]++;}}
  array<ll,26>At,Bt;for(int c=0;c<26;c++){ll s1=0,s2=0;for(int x:b){if(c<x)s1+=v(x);if(x<c)s2+=v(c);}At[c]=s1;Bt[c]=s2;}
  ll Ktt=K(ct,ct);
  int Q=n-m+1;vector<ull>h(n+1,0),pw(n+1,1);for(int i=0;i<n;i++){h[i+1]=(mulm(h[i],BASE)+a[i]+1)%MOD;pw[i+1]=mulm(pw[i],BASE);}
  auto sub=[&](int i){return (h[i+m]+MOD-mulm(h[i],pw[m]))%MOD;};
  vector<pair<ull,int>>hs(Q);for(int i=0;i<Q;i++)hs[i]={sub(i),i};sort(hs.begin(),hs.end());
  vector<int>nxt(Q,-1);for(int lo=0;lo<Q;){int hi=lo;while(hi<Q&&hs[hi].first==hs[lo].first)hi++;int p2=lo;for(int k=lo;k<hi;k++){int q=hs[k].second;while(p2<hi&&hs[p2].second<q+m)p2++;nxt[q]=p2<hi?hs[p2].second:-1;}lo=hi;}
  vector<ll>sumD(Q+1,0),cnt(Q+1,0);ll best=LLONG_MAX;
  for(int q=Q-1;q>=0;q--){
   array<ll,26>cp;for(int c=0;c<26;c++)cp[c]=pre[q+m][c]-pre[q][c];
   ll Ip;{ll w=0,acc=0;array<ll,26>sv;for(int c=25;c>=0;c--){sv[c]=acc;acc+=v(c)*cp[c];}for(int c=0;c<26;c++)w+=(ll)pre[q][c]*sv[c];Ip=(Lp[q+m]-Lp[q])-w;}
   ll newOut=0;for(int c=0;c<26;c++)newOut+=(ll)pre[q][c]*At[c]+(ll)(pre[n][c]-pre[q+m][c])*Bt[c];
   ll oldOut=(LR[q+m]-LR[q])-2*Ip;
   ll d=newOut-oldOut;int nq=nxt[q];sumD[q]=d+(nq>=0?sumD[nq]:0);cnt[q]=1+(nq>=0?cnt[nq]:0);
   ll r=cnt[q];ll Kpp=K(cp,cp),Ktp=K(ct,cp),Kpt=K(cp,ct);
   ll Vp=0,Vt=0;for(int c=0;c<26;c++){Vp+=v(c)*cp[c];Vt+=v(c)*ct[c];}
   ll delta=sumD[q]+r*(It-Ip)+r*(Vt-Vp)+r*(r-1)/2*(Ktt-Ktp-Kpt+Kpp);best=min(best,S0+delta);}
  printf("%lld\n",best);}}
