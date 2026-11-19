#include <bits/stdc++.h>
using namespace std;
int main(){int T;scanf("%d",&T);while(T--){int n;long long k;scanf("%d %lld",&n,&k);vector<long long>m(n),c(n);for(int i=0;i<n;i++)scanf("%lld %lld",&m[i],&c[i]);
 // byM: suffix max of c over people sorted by m; byC: suffix max of m over people sorted by c.
 vector<int>a(n),b(n);iota(a.begin(),a.end(),0);b=a;sort(a.begin(),a.end(),[&](int x,int y){return m[x]<m[y];});sort(b.begin(),b.end(),[&](int x,int y){return c[x]<c[y];});
 vector<long long>am(n),bc(n),sc(n+1,LLONG_MIN),sm(n+1,LLONG_MIN);for(int i=0;i<n;i++){am[i]=m[a[i]];bc[i]=c[b[i]];}
 for(int i=n-1;i>=0;i--){sc[i]=max(sc[i+1],c[a[i]]);sm[i]=max(sm[i+1],m[b[i]]);}
 vector<int>res;for(int i=0;i<n;i++){int p=lower_bound(am.begin(),am.end(),m[i]+k)-am.begin();bool bad=sc[p]>=c[i];int q=lower_bound(bc.begin(),bc.end(),c[i]+k)-bc.begin();if(sm[q]>=m[i])bad=true;if(!bad)res.push_back(i+1);}
 printf("%zu\n",res.size());for(size_t i=0;i<res.size();i++)printf("%d%c",res[i],i+1==res.size()?'\n':' ');}}
