#include <bits/stdc++.h>
using namespace std;
// Two neighbours can both be jumped unless each splashes the other, so the answer
// is a maximum independent set over runs of mutually splashing neighbours.
int main(){int T;scanf("%d",&T);while(T--){int n;scanf("%d",&n);static char L[200005],R[200005];scanf("%s %s",L,R);long long ans=0;int i=0;while(i<n){int j=i;while(j<n-1&&L[j]=='1'&&R[j]=='1')j++;ans+=(j-i+2)/2;i=j+1;}printf("%lld\n",ans);}}
