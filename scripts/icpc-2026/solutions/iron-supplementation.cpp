#include <bits/stdc++.h>
using namespace std;
int main(){int T;cin>>T;cout<<fixed<<setprecision(12);while(T--){int m,d,p;cin>>m>>d>>p;long double x=0;for(int i=0;i<d;i++){x+=m;x*=(100-p)/100.0L;}cout<<(double)x<<"\n";}}
