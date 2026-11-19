#include <bits/stdc++.h>
using namespace std;
int main(){int T;cin>>T;while(T--){long long n;int k;cin>>n>>k;set<long long>s{n};for(int i=0;i<k;i++){set<long long>t;for(long long x:s){t.insert(x/2);t.insert((x+1)/2);}s=t;}cout<<s.size()<<"\n";}}
