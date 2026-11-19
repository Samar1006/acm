#include <bits/stdc++.h>
using namespace std;
// Cables are lines through columns (x) and rows (y). A horizontal and a vertical line always
// cross, so the answer is a minimum vertex cover of the column-row graph (points are edges)
// that uses both orientations: the matching size M if such a minimum cover exists, else M + 1.
// One line suffices when all points share a row or a column.
int nc,nr;vector<vector<int>>adj;vector<int>mc,mr,dist_;
bool bfs(){queue<int>q;bool f=false;for(int c=0;c<nc;c++){if(mc[c]<0){dist_[c]=0;q.push(c);}else dist_[c]=-1;}
 while(!q.empty()){int c=q.front();q.pop();for(int r:adj[c]){int c2=mr[r];if(c2<0)f=true;else if(dist_[c2]<0){dist_[c2]=dist_[c]+1;q.push(c2);}}}return f;}
bool dfs(int c){ // iterative augmenting path search
 vector<pair<int,size_t>>st{{c,0}};vector<int>path;
 while(!st.empty()){auto&[u,i]=st.back();if(i==adj[u].size()){dist_[u]=-1;st.pop_back();if(!path.empty()&&path.size()>st.size())path.pop_back();continue;}
  int r=adj[u][i++];int c2=mr[r];
  if(c2<0||(dist_[c2]==dist_[u]+1)){
   if(c2<0){ // augment along stack
    vector<int>rows;rows.push_back(r);for(int k=(int)st.size()-1;k>=0;k--){int cc=st[k].first;int rr=rows.back();int old=mc[cc];mc[cc]=rr;mr[rr]=cc;if(k>0)rows.push_back(old);}
    return true;}
   st.push_back({c2,0});}}
 return false;}
int main(){int T;scanf("%d",&T);while(T--){int n;scanf("%d",&n);vector<long long>X(n),Y(n);for(int i=0;i<n;i++)scanf("%lld %lld",&X[i],&Y[i]);
 vector<long long>xs(X),ys(Y);sort(xs.begin(),xs.end());xs.erase(unique(xs.begin(),xs.end()),xs.end());sort(ys.begin(),ys.end());ys.erase(unique(ys.begin(),ys.end()),ys.end());
 nc=xs.size();nr=ys.size();if(nc==1||nr==1){puts("1");continue;}
 adj.assign(nc,{});vector<pair<int,int>>E(n);for(int i=0;i<n;i++){int c=lower_bound(xs.begin(),xs.end(),X[i])-xs.begin(),r=lower_bound(ys.begin(),ys.end(),Y[i])-ys.begin();adj[c].push_back(r);E[i]={c,r};}
 mc.assign(nc,-1);mr.assign(nr,-1);dist_.assign(nc,0);int M=0;while(bfs())for(int c=0;c<nc;c++)if(mc[c]<0&&dfs(c))M++;
 vector<vector<int>>radj(nr);for(auto[c,r]:E)radj[r].push_back(c);
 // D: reachable from a free vertex by alternating paths (never in a minimum cover);
 // A: their neighbours (in every minimum cover).
 vector<char>Dc(nc,0),Dr(nr,0),Ac(nc,0),Ar(nr,0);
 {queue<int>q;for(int c=0;c<nc;c++)if(mc[c]<0){Dc[c]=1;q.push(c);}while(!q.empty()){int c=q.front();q.pop();for(int r:adj[c]){Ar[r]=1;int c2=mr[r];if(c2>=0&&!Dc[c2]){Dc[c2]=1;q.push(c2);}}}}
 {queue<int>q;for(int r=0;r<nr;r++)if(mr[r]<0){Dr[r]=1;q.push(r);}while(!q.empty()){int r=q.front();q.pop();for(int c:radj[r]){Ac[c]=1;int r2=mc[c];if(r2>=0&&!Dr[r2]){Dr[r2]=1;q.push(r2);}}}}
 bool hasAc=false,hasAr=false;for(int c=0;c<nc;c++)hasAc|=Ac[c];for(int r=0;r<nr;r++)hasAr|=Ar[r];
 vector<int>Cc;vector<int>idx(nc,-1);for(int c=0;c<nc;c++)if(mc[c]>=0&&!Dc[c]&&!Ac[c]&&!Dr[mc[c]]&&!Ar[mc[c]]){idx[c]=Cc.size();Cc.push_back(c);}
 bool mixed=false;
 if(hasAc&&hasAr)mixed=true;else if((hasAc||hasAr)&&!Cc.empty())mixed=true;
 else if(!hasAc&&!hasAr&&!Cc.empty()){
  // Inside the perfectly matched part, choosing column c' forces column c for every point (c, mate(c')).
  int K=Cc.size();vector<vector<int>>g(K),rg(K);
  for(auto[c,r]:E){if(idx[c]<0||mr[r]<0||idx[mr[r]]<0||mr[r]==c)continue;int a=idx[mr[r]],b=idx[c];g[a].push_back(b);rg[b].push_back(a);}
  vector<int>order;vector<char>vis(K,0);for(int s=0;s<K;s++){if(vis[s])continue;vector<pair<int,size_t>>st{{s,0}};vis[s]=1;while(!st.empty()){auto&[u,i]=st.back();if(i<g[u].size()){int v=g[u][i++];if(!vis[v]){vis[v]=1;st.push_back({v,0});}}else{order.push_back(u);st.pop_back();}}}
  vector<int>comp(K,-1);int nscc=0;for(int k=K-1;k>=0;k--){int s=order[k];if(comp[s]>=0)continue;vector<int>st{s};comp[s]=nscc;while(!st.empty()){int u=st.back();st.pop_back();for(int v:rg[u])if(comp[v]<0){comp[v]=nscc;st.push_back(v);}}nscc++;}
  mixed=nscc>=2;}
 printf("%d\n",mixed?M:M+1);}}
