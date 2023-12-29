#include <iostream>
#include <fstream>
#include <map>
#include <vector>
#include <boost/algorithm/string.hpp>

using namespace std;
struct Syncer{	
	int  Port;
	string Ips;	
};
struct Bunny{
	vector <string> m_front;
	vector <string> t_front;
	// vector <string> notify;
	// vector <string> sync;
	map<string,  vector<Syncer>  > sync;
	int  port;
	string type;
	string authcode;
	string appid;
	string user;
	string passwd;
	string brokerid;
	// vector <string> subscribe;
	friend std::ostream& operator<<(std::ostream& os, const Bunny& obj){
		os<<"Bunny is \n";
		os <<"\ttype: "<< obj.type <<"\n";
		os<<"\tuser: "<< obj.user <<"\n";
		os<<"\tpasswd: "<< obj.passwd <<"\n";
		os<<"\tappid: "<< obj.appid <<"\n";
		os<<"\tbrokerid: "<< obj.brokerid <<"\n";
		os<<"\tauthcode: "<< obj.authcode <<"\n";
		os<<"\tmd_front:\n";
		for (auto &x:obj.m_front){
			os<<"\t\t"<< x <<"\n";
		}
		os<<"\n\ttb_front:\n";
		for (auto &x:obj.t_front){
			os<<"\t\t"<< x <<"\n";
		}
		/*
		os<<"\n\tsubscribe:\n";
		for (auto &x:obj.subscribe){
			os<<"\t\t"<< x <<"\n";
		} 
		os<<"\n\tnotify:\n";
		for (auto &x:obj.notify){
			os<<"\t\t"<< x <<"\n";
		} */
		os<<"\n\tsync:\n";
		for (auto &x:obj.sync){
			os << "\t\t"<< x.first << ":\n";
			for (auto &y:x.second)
				os << "\t\t\t"  << y.Ips << ":" << y.Port << "\n";
		}
		return os;
	}

};

std::string& trim(std::string &s);
int readconfig(const char *fname,  map<string, string> &mconfig);
bool build_bunny(string &key, string &val, Bunny &bn);
vector<string> stringSplit(const string& str, char delim);
string& trim(string &s);

//////////////////////////
///
// 读取文件，以首个'='作为分界符，生成key,value的 map结构。
// 不识别重复情况！！！！！！！
int readconfig(const char *fname,  map<string, string> &mconfig)
{
	ifstream fin;
	fin.open(fname, ios::in);
	if (!fin.is_open())
	{
		cout << "无法找到这个文件！" << endl;
		return 0;
	}
	string buff;
	while (getline(fin, buff))
	{
//		cout << buff<<endl;
		trim(buff);
		if (buff.find("#") == 0){
			// cout << "注释："<< buff << endl;
			continue;
		}else{
			int idx = buff.find("=");
			if (idx > 0) {
				string key = buff.substr(0,idx);				
				string val = buff.substr(idx+1, buff.length()-idx-1);
				trim(key);
				trim(val);
				// cout << key << " = " << val << endl;
		 		mconfig.insert(std::pair<string, string>(key, val));
			}else{
				// cout <<"Not find '=' in:"<<buff<< endl;
			}
					
		}

	}
	fin.close();
	return 0;
}

vector<string> stringSplit(const std::string& str, char delim) {
    std::size_t previous = 0;
    std::size_t current = str.find_first_of(delim);
    vector<string> elems;
    while (current != std::string::npos) {
        if (current > previous) {
            string s = str.substr(previous, current - previous);
	    trim(s);
	    if (s.length() > 0){

		    elems.push_back(s);
	    }	
	}
        previous = current + 1;
        current = str.find_first_of(delim, previous);
    }
    if (previous != str.size()) {
	    string s = str.substr(previous);
	    trim(s);
	    if (s.length() > 0){

		    elems.push_back(s);
	    }	
    }
    return elems;
}

bool build_bunny(string &key, string &val, Bunny &bn){

	if ( boost::iequals(key.c_str(), "authcode") ){
		bn.authcode = val;
	}else if ( boost::iequals(key.c_str(), "type")){
		bn.type = val;
	}else if ( boost::iequals(key.c_str(), "appid")){
		bn.appid = val;
	}else if ( boost::iequals(key.c_str(), "brokerid") ){
		bn.brokerid = val;
	}else if( boost::iequals(key.c_str(), "passwd")){
		bn.passwd = val;
	}else if ( boost::iequals(key.c_str(), "user") ){
		bn.user = val;
	}else if ( boost::iequals(key.c_str(), "m_front") ){
		bn.m_front = stringSplit(val,',');		
	}else if ( boost::iequals(key.c_str(), "t_front") ){
		bn.t_front = stringSplit(val,',');	
	/*	
	}else if ( boost::iequals(key.c_str(), "notify") ){
		bn.notify = stringSplit(val,','); */		
	}else if ( boost::iequals(key.c_str(), "sync") ){
		vector <string> sv = stringSplit(val,',');		
		for (auto &x:sv){
			vector <string> sv2 = stringSplit(x,':');
			Syncer s;
			s.Port = atoi(sv2[2].c_str());
			s.Ips = sv2[1];
			if (bn.sync.end() != bn.sync.find(sv2[0] )){
				bn.sync[sv2[0]].push_back(s);
			}else{
				vector<Syncer> v;
				v.push_back(s);
				bn.sync.insert(std::pair<string, vector<Syncer> >(sv2[0], v));
			}
		}
	}
	/*
	else if ( boost::iequals(key, "subscribe") ){
		bn.subscribe = stringSplit(val,',');	}
		*/
	else if ( boost::iequals(key, "port") ){
		bn.port = atoi(val.c_str());	
	}
	else{
		cout <<"error:"<< key << endl;
		return false;
	}
	return true;
}


std::string& trim(std::string &s)
{
    if (s.empty())
    {
        return s;
    }
    s.erase(0,s.find_first_not_of(" "));
    s.erase(0,s.find_first_not_of("\t"));
    s.erase(s.find_last_not_of(" ") + 1);
    s.erase(s.find_last_not_of("\t") + 1);
    return s;
}

/*
int main()
{

	map<string, string> mconfig;
	readconfig("./abc.txt", mconfig );

	Bunny bn;
	for (auto& x: mconfig) {
	// cout << x.first << ":" << x.second << endl;
	    bool b = build_bunny((string &)x.first,(string &)x.second, bn);		
	    if (!b){
		cout << "CAN'T PROCESS "<< x.first << endl;
	    }
	}
	cout << "Now bunny is:"<<endl;	
	cout << "user = "<<bn.user <<";  passwd = "<<bn.passwd<<endl;

	cout << bn << endl;
	return 0;

}
*/
