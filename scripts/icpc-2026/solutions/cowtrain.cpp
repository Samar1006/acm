#include <bits/stdc++.h>
using namespace std;
int main(){int T;cin>>T;while(T--){string s,t;cin>>s>>t;map<char,char>f,g;bool ok=true;for(size_t i=0;i<s.size();i++){auto a=f.find(s[i]);if(a!=f.end()&&a->second!=t[i])ok=false;auto b=g.find(t[i]);if(b!=g.end()&&b->second!=s[i])ok=false;f[s[i]]=t[i];g[t[i]]=s[i];}cout<<(ok?"YES":"NO")<<"\n";}}
