#include <iostream>
#include <vector>
#include <chrono>
#include <map>
#include <sstream>

using namespace std;

#ifndef _STRATEGY_H
#define _STRATEGY_H
#endif

class TopVolume
{
public:
	TopVolume(int n, int m = 50) : m_num(n), m_min(m){}; // 拷贝构造函数
	TopVolume(const TopVolume &obj)
	{
		this->m_num = obj.m_num;
		this->m_topV = obj.m_topV;
		this->m_topT = obj.m_topT;
	};				// 拷贝构造函数
	~TopVolume(){}; // 这是析构函数声明

	bool insert(int vol, string st);
	bool is_top(int vol);
	void print();
	void clear();

private:
	int m_num;
	int m_min;
	map<int, string> m_topV;
	map<string, int> m_topT;
};

enum TradeType
{
	KDUO,
	KKONG,
	PDUO,
	PKONG,
	FAN
};
string trade_string(const TradeType &obj);


enum StrategyType
{
	MACD,
	VOLUME,
	NONE
};

struct Trade
{
	int Volume;
	double Price;
	string Time;
	TradeType ttype;
};

class Strategy
{
public:
	Strategy(int n) : m_num(n){}; // 拷贝构造函数
	Strategy(const Strategy &obj)
	{
		this->m_num = obj.m_num;
		this->m_records = obj.m_records;
	};			   // 拷贝构造函数
	~Strategy(){}; // 这是析构函数声明

	bool Sell(double p, string t);
	bool Buy(double p, string t);
	bool Fan(double p, string t);
	bool HighAction(double p, string t);
	bool LowAction(double p, string t);
	void Print();
	void Clear();
	double Calculate();

private:
	// count every times
	int m_num;
	vector<Trade> m_records;
};


struct AVsignal
{
	int signal;	
	double Price;
	string Time;	
};

class AVStrategy
{
public:
	AVStrategy(double val) : m_Value(val){  if (val == 0.0)  m_Value = 1.0; }; // 拷贝构造函数
	
	~AVStrategy(){}; // 这是析构函数声明

	bool Next(double now, double av, string st);
	int GetSignal(int idx = -1);
	void Clear();

private:
	// 
	int m_Value;
	vector< AVsignal > m_Avs;
};