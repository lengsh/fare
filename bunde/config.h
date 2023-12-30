#include <iostream>
#include <fstream>
#include <map>
#include <vector>
#include <boost/algorithm/string.hpp>
#include "strategy.h"

using namespace std;

#define TOP_N 20
#define RSI_PERIOD 10
#define VOLUME_BAR 6
#define VOLUME_EMA_SMALL 6
#define VOLUME_EMA_BIG 18
#define MACD_EMA_SMALL 12
#define MACD_EMA_BIG 26
#define BORDER_WIDTH 10

#ifdef MACOS
#define WIN_WIDTH 2600
#define WIN_HIGH 1200
#define WIN_INDICATOR 340
#define RIGHT_WTH 200	// 右侧信息栏宽度
#define T_MACD_LEFT 180 // 左侧MACD定位
#define FONT_FILE "/System/Library/Fonts/Supplemental/Times New Roman.ttf"
#else
#define WIN_WIDTH 1680
#define WIN_HIGH 800
#define WIN_INDICATOR 250
#define RIGHT_WTH 200	// 右侧信息栏宽度
#define T_MACD_LEFT 140 // 左侧MACD定位
#define FONT_FILE "/usr/share/fonts/truetype/xingshu.ttf"
#endif

#ifdef MACOS
#define F_SIZE_TITLE 26
#define F_SIZE_TOPIC 20
#define F_SIZE_INFO 18
#define F_SIZE_BOARD 16
#define F_SIZE_TIME 16
#else
#define F_SIZE_TITLE 26
#define F_SIZE_TOPIC 22
#define F_SIZE_INFO 20
#define F_SIZE_BOARD 18
#define F_SIZE_TIME 16
#endif

struct Bunny
{
	string notify;		   // xman的通知POST API，如：http://127.0.0.1:8080/xman/sendmessage
	int port;			   // 行情监听端口
	double dvalue;		   // 价格差价触发阈值
	int bufsize;		   // 不需要设置，自动计算；
	int width;			   // 不需要设置，自己计算
	int high;			   // 窗口高
	float barsize;		   // 每个交易记录点占用多少个像素
	float volscale;		   // 交易量缩放比例
	int volwarning;		   // 交易量预警阈值
	string fName;		   // 商品编号
	string fFlag;		   // 图标文件路径和名称
	string bunny;		   // 远程服务
	bool seconds;		   // 按秒记录， true 则是，否则按10秒记
	float av_args;             // 均值参数         
	StrategyType strategy; // macd, volume, none

	friend std::ostream &operator<<(std::ostream &os, const Bunny &obj)
	{
		os << "Bunny is \n";
		os << "\n\tport: " << obj.port;
		os << "\n\tfName: " << obj.fName;
		os << "\n\tbunny: " << obj.bunny;
		os << "\n\tfFlag: " << obj.fFlag;
		os << "\n\tdvalue: " << to_string(obj.dvalue);
		os << "\n\tvolwarning: " << to_string(obj.volwarning);
		os << "\n\tbufsize: " << obj.bufsize;
		os << "\n\twidth: " << obj.width;
		os << "\n\thigh: " << obj.high;
		os << "\n\tbarsize: " << obj.barsize;
		os << "\n\tvolscale: " << obj.volscale;
		os << "\n\tav_args: " << obj.av_args;

		if (obj.seconds)
			os << "\n\tseconds: true";
		else
			os << "\n\tseconds: false";
		os << "\n\tnotify:" << obj.notify;
		if (obj.strategy == StrategyType::MACD)
			os << "\n\tstrategy: macd\n";
		else if (obj.strategy == StrategyType::VOLUME)
			os << "\n\tstrategy: volume\n";
		else
			os << "\n\tstrategy: none\n";
		return os;
	}
};

std::string &trim(std::string &s);
int readconfig(const char *fname, map<string, string> &mconfig);
bool build_bunny(string &key, string &val, Bunny &bn);
vector<string> stringSplit(const string &str, char delim);
string &trim(string &s);

//////////////////////////
///
// 读取文件，以首个'='作为分界符，生成key,value的 map结构。
// 不识别重复情况！！！！！！！
int readconfig(const char *fname, map<string, string> &mconfig)
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
		if (buff.find("#") == 0)
		{
			// cout << "注释："<< buff << endl;
			continue;
		}
		else
		{
			int idx = buff.find("=");
			if (idx > 0)
			{
				string key = buff.substr(0, idx);
				string val = buff.substr(idx + 1, buff.length() - idx - 1);
				trim(key);
				trim(val);
				// cout << key << " = " << val << endl;
				mconfig.insert(std::pair<string, string>(key, val));
			}
			else
			{
				// cout <<"Not find '=' in:"<<buff<< endl;
			}
		}
	}
	fin.close();
	return 0;
}

vector<string> stringSplit(const std::string &str, char delim)
{
	std::size_t previous = 0;
	std::size_t current = str.find_first_of(delim);
	vector<string> elems;
	while (current != std::string::npos)
	{
		if (current > previous)
		{
			string s = str.substr(previous, current - previous);
			trim(s);
			if (s.length() > 0)
			{

				elems.push_back(s);
			}
		}
		previous = current + 1;
		current = str.find_first_of(delim, previous);
	}
	if (previous != str.size())
	{
		string s = str.substr(previous);
		trim(s);
		if (s.length() > 0)
		{

			elems.push_back(s);
		}
	}
	return elems;
}

bool build_bunny(string &key, string &val, Bunny &bn)
{
	if (boost::iequals(key.c_str(), "port"))
	{
		bn.port = atoi(val.c_str());
	}
	else if (boost::iequals(key.c_str(), "bunny"))
	{
		bn.bunny = val.c_str();
	}
	else if (boost::iequals(key.c_str(), "fName"))
	{
		bn.fName = val;
	}
	else if (boost::iequals(key.c_str(), "strategy"))
	{
		if (val.compare("macd") == 0)
			bn.strategy = StrategyType::MACD;
		else if (val.compare("volume") == 0)
			bn.strategy = StrategyType::VOLUME;
		else
			bn.strategy = StrategyType::NONE;
	}
	else if (boost::iequals(key.c_str(), "notify"))
	{
		bn.notify = val;
	}
	else if (boost::iequals(key.c_str(), "fFlag"))
	{
		bn.fFlag = val;
	}
	else if (boost::iequals(key.c_str(), "seconds"))
	{
		bn.seconds = val.compare("true") == 0 ? true : false;
	}
	else if (boost::iequals(key.c_str(), "av_args"))
	{
		bn.av_args = atof(val.c_str());
	}
	else if (boost::iequals(key.c_str(), "barsize"))
	{
		bn.barsize = atof(val.c_str());
	}
	else if (boost::iequals(key.c_str(), "volscale"))
	{
		bn.volscale = atof(val.c_str());
	}
	else if (boost::iequals(key.c_str(), "volwarning"))
	{
		bn.volwarning = atoi(val.c_str());
	}
	else if (boost::iequals(key.c_str(), "high"))
	{
		bn.high = atoi(val.c_str());
	}
	else if (boost::iequals(key.c_str(), "dvalue"))
	{
		bn.dvalue = atof(val.c_str());
	}
	else
	{
		cout << "error:" << key << endl;
		return false;
	}
	return true;
}

std::string &trim(std::string &s)
{
	if (s.empty())
	{
		return s;
	}
	s.erase(0, s.find_first_not_of(" "));
	s.erase(0, s.find_first_not_of("\t"));
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
