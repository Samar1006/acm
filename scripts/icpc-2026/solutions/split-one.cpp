#include <bits/stdc++.h>
using namespace std;
int main(){int T;cin>>T;cout<<fixed<<setprecision(9);while(T--){int n;cin>>n;long long pd=0,pt=0;double ans=-1;for(int i=0;i<n;i++){long long d,t;cin>>d>>t;if(ans<0&&d>=5280)ans=pt+(double)(t-pt)*(5280-pd)/(d-pd);pd=d;pt=t;}cout<<ans<<"\n";}}
