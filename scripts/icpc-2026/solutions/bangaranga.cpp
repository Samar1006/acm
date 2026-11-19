#include <bits/stdc++.h>
using namespace std;
int main(){int T;cin>>T;while(T--){long long d;int q;cin>>d>>q;while(q--){string op,k;cin>>op>>k;long long e=10LL*(long long)(k.size()-1);d+=op=="BANGA"?e:-e;cout<<d<<"\n";}}}
