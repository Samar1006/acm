#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
int main(){int T;scanf("%d",&T);while(T--){int n;ll p;scanf("%d %lld",&n,&p);vector<ll>j(n),t(n);for(int i=0;i<n;i++)scanf("%lld %lld",&j[i],&t[i]);
 // Before c announcements: totals of the first c are known; R points remain for the rest.
 ll best=LLONG_MIN,second=LLONG_MIN;int bi=-1;ll R=p;int ans=-1;
 for(int c=0;c<=n;c++){
  if(c>0){ll a=j[c-1]+t[c-1];R-=t[c-1];if(a>best){second=best;best=a;bi=c-1;}else if(a>second)second=a;}
  int rest=n-c;bool ok=false;
  // An announced leader wins if nobody announced ties it and no unannounced team can reach it.
  if(c>0&&best>second){if(rest==0)ok=true;else if(j[n-1]+R<best)ok=true;}
  // The last team wins without votes only if it beats everyone even when another team gets all R.
  if(!ok&&rest>=1){ll jn=j[n-1];if(rest==1){if(c==0||jn+R>best)ok=true;}else{if((c==0||jn>best)&&jn>j[n-2]+R)ok=true;}}
  if(ok){ans=c;break;}}
 printf("%d\n",ans);}}
