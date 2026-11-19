#include <bits/stdc++.h>
using namespace std;
typedef long double ld;typedef long long ll;
// Per task: rounds until a non-failure cost T = t(s+f+c)/(s+c) and succeed with q = s/(s+c).
// A corruption restarts everything, so the best plan is a fixed order and
// E = sum_k T_k prod_{j<k} q_j / prod q. Exchange argument: sort by T/(1-q).
int main(){int T;scanf("%d",&T);while(T--){int n;scanf("%d",&n);vector<ld>tt(n),q(n);bool imp=false;
 for(int i=0;i<n;i++){ll t,s,f,c;scanf("%lld %lld %lld %lld",&t,&s,&f,&c);if(s==0){imp=true;continue;}tt[i]=(ld)t*(s+f+c)/(s+c);q[i]=(ld)s/(s+c);}
 if(imp){puts("impossible");continue;}
 vector<int>o(n);iota(o.begin(),o.end(),0);sort(o.begin(),o.end(),[&](int a,int b){return tt[a]*(1-q[b])<tt[b]*(1-q[a]);});
 ld sum=0,pr=1;for(int i:o){sum+=tt[i]*pr;pr*=q[i];}printf("%.12Le\n",sum/pr);}}
