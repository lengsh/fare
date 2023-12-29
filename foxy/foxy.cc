#include <iostream>
#include <chrono>
#include <thread>
#include <time.h>
#include <sys/select.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <cstdlib>
#include <cstdio>

#ifdef _WIN32
#include "win/getopt.h"
#else
#include <unistd.h>
#endif
#include "ThostFtdcMdApi.h"
#include "ThostFtdcTraderApi.h"
#include <cstring>
#include "httpclient.h"
#include "config.h"
#include "crow.h"
// #include <iomanip>
#include <sstream>

using namespace std;
using namespace std::chrono;

auto reqtime = std::chrono::steady_clock::now();
Bunny g_Config;
// 要订阅的合约列表
char **instruments = NULL;
size_t instrument_count;
bool GetDepthMarketDataJson(CThostFtdcDepthMarketDataField *pd, string &resp);
string vect2string(vector<string> &v);
void sync_to_monitor(int sockfd, const char *name, const char *src, int len);

template <typename T>
int structToBuf(const T &structObj, void *buf)
{
	const int size = sizeof(T);
	memcpy(buf, &structObj, size);
	return size;
}

template <typename T>
T bufToStruct(void *buf)
{
	const int size = sizeof(T);
	T structObj;
	memcpy(&structObj, buf, size);
	return structObj;
}

int g_Counter;

class CMarketSpi : public CThostFtdcMdSpi
{
private:
	HttpClient hc;
	int sockfd;

public:
	CMarketSpi(CThostFtdcMdApi *pApi) : m_pApi(pApi)
	{
		// 创建socket
		this->sockfd = socket(AF_INET, SOCK_DGRAM, 0);
		if (-1 == this->sockfd)
		{
			throw("can't create socket");
		}
		pApi->RegisterSpi(this);
	}

	~CMarketSpi()
	{
		close(this->sockfd);
	}

	void OnFrontConnected()
	{
		std::cout << "connected." << std::endl;
		reqtime = std::chrono::steady_clock::now();
		CThostFtdcReqUserLoginField Req;
		memset(&Req, 0x00, sizeof(Req));
		strncpy(Req.UserID, "218695", sizeof(Req.UserID) - 1);
		strncpy(Req.Password, "x94N2ewEHq!", sizeof(Req.Password) - 1);
		strncpy(Req.BrokerID, "9999", sizeof(Req.BrokerID) - 1);

		m_pApi->ReqUserLogin(&Req, 0);
	}

	void OnFrontDisconnected(int nReason)
	{
		std::cout << "disconnected." << std::endl;
	}

	void OnRspUserLogin(CThostFtdcRspUserLoginField *pRspUserLogin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
	{
		auto rsptime = std::chrono::steady_clock::now();
		auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(rsptime - reqtime);

		std::cout << "response time: " << duration.count() << " milliseconds" << std::endl;
		m_pApi->SubscribeMarketData(instruments, instrument_count);
	}

	// 订阅行情应答
	void OnRspSubMarketData(CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
	{

		if (pRspInfo->ErrorID == 0)
		{
			cout << "订阅 " << pSpecificInstrument->InstrumentID << "  行情数据成功！" << endl;
		}
		else
		{
			cout << "订阅 " << pSpecificInstrument->InstrumentID << "  行情数据失败！" << endl;

			string err(pRspInfo->ErrorMsg);
			cout << "ErrorID=" << pRspInfo->ErrorID << ": (gbk信息显示失败，待转成utf8)" << err << endl;
		}
	}

	void OnRtnDepthMarketData(CThostFtdcDepthMarketDataField *pDepthMarketData)
	{
		/*
		cout << "上次结算价   = " << pDepthMarketData->PreSettlementPrice << "\n";
		cout << "昨持仓量     = " << pDepthMarketData->PreOpenInterest << "\n";
		cout << "持仓量       = " << pDepthMarketData->OpenInterest << "\n";
		cout << "昨持仓量     = " << pDepthMarketData->PreOpenInterest << "\n";
		cout << "成交价格     = " << pDepthMarketData->LastPrice << "\n";
		cout << "累积成交数量 = " << pDepthMarketData->Volume << "\n";
		cout << "累积成交金额 = " << pDepthMarketData->Turnover << endl;
		*/
		if (g_Config.sync.size() > 0)
		{
			if (g_Config.sync.end() != g_Config.sync.find(pDepthMarketData->InstrumentID))
			{
				char buf[1024];
				// int len = structToBuf<CThostFtdcDepthMarketDataField>(*pDepthMarketData, buf);
				//
				// 为了兼容rust程序，改成如下，变量按字母顺序排序；
				// 卖出
				int sell = pDepthMarketData->AskVolume1 + 
					pDepthMarketData->AskVolume2 + 
					pDepthMarketData->AskVolume3 +
					pDepthMarketData->AskVolume4 +
					pDepthMarketData->AskVolume5;					
				// 买入
				int buy = pDepthMarketData->BidVolume1 + 
					pDepthMarketData->BidVolume2 + 
					pDepthMarketData->BidVolume3 +
					pDepthMarketData->BidVolume4 +
					pDepthMarketData->BidVolume5;
				// int buyOrsell = bidV - askV; //  bidV > askV ? 1 : -1;
				// 9 个 变量！！！
				int len = snprintf(buf, 1024, "%.2f;%d;%d;%s;%.2f;%.2f;%s;%s;%d", 
						pDepthMarketData->AveragePrice,
						buy,
						sell,
						pDepthMarketData->InstrumentID,	
						pDepthMarketData->LastPrice,
						pDepthMarketData->OpenPrice,
						pDepthMarketData->TradingDay,
						pDepthMarketData->UpdateTime,
						pDepthMarketData->Volume);
					
				sync_to_monitor(this->sockfd, pDepthMarketData->InstrumentID, buf, len);
			}
		}
		/*
		float yestoday = pDepthMarketData->PreSettlementPrice * pDepthMarketData->PreOpenInterest;
		float today = 0.0;
		if (pDepthMarketData->Volume > 0)
		{
			today = pDepthMarketData->OpenInterest * (pDepthMarketData->Turnover / float(pDepthMarketData->Volume));
		}

		float current = today - yestoday;
		cout << pDepthMarketData->InstrumentID << "资金额 =(成交金额/数量)*持仓量 -  上次结算价*昨持仓量 = " << current << endl;
		*/
		// HttpClient hc;
		auto t = system_clock::now();
		auto t_now = system_clock::to_time_t(t);
		std::tm tm = {0};
		localtime_r(&t_now, &tm); // linux线程安全, windows is localtime_t()
		char now_str[32];
		// 	strftime(now_str, 32, "%Y%m%d %H:%M:%S", &tm);
		// cout << "Now is "<< now_str<< "; the ticks is "<< pDepthMarketData->TradingDay<<" "<< pDepthMarketData->UpdateTime << endl;
		strftime(now_str, 32, "%Y%m%d", &tm);
		string today_s(now_str);

		if (g_Counter%200 == 1){
			char now_str[32];		
			strftime(now_str, 32, "%Y%m%d %H:%M:%S", &tm);
			char time_str[32];
			strftime(time_str, 32, "%Y%m%d  %H:%M:%S", &tm);
			int package = pDepthMarketData->AveragePrice / pDepthMarketData->LastPrice;
			cout <<time_str << ",	TradingDay = " << pDepthMarketData->TradingDay << ", ";
			cout << "InstrumentID = " << pDepthMarketData->InstrumentID << ", ";
			cout << pDepthMarketData->LastPrice << ", "<< pDepthMarketData->AveragePrice << ", package ="<< package << " or "<< package + 1 << "\n";
			/*
			if (today_s.compare(pDepthMarketData->TradingDay) != 0)
			{
				cout << "NoT today, Now is " << today_s << endl;
				// return;
			} */
		}
		g_Counter = (g_Counter + 1)%200;
		/*
		if (g_Config.notify.size() > 0 )
		{
			string resp;
			string json;
			GetDepthMarketDataJson(pDepthMarketData, json);
			// cout << json << endl;
			for (int i = 0; i < g_Config.notify.size(); i++)
			{
				// cout << "post to " << g_Config.notify[i] << endl;
				/////////////////////////////////////////////////////
				// remove
				// this->hc.Post((char *)bn.notify[i].c_str(), json, resp);
			}
		}
		else
		{
			// cout << "No notify" << endl;
		}
		*/
	}

	CThostFtdcMdApi *m_pApi;
};

void print_usage()
{
	std::cout << "example:\nbunny config_tts.txt|config_ctp.txt" << std::endl;
}

bool GetDepthMarketDataJson(CThostFtdcDepthMarketDataField *pd, string &resp)
{
	resp.clear();
	// 交易日
	resp.append("{\"tradingDay\":\"");
	resp.append(pd->TradingDay);
	resp.append("\",\"exchangeID\":\"");
	resp.append(pd->ExchangeID);
	resp.append("\",\"highestPrice\":");
	resp.append(to_string(pd->HighestPrice));
	resp.append(",\"lowestPrice\":");
	resp.append(to_string(pd->LowestPrice));
	TThostFtdcPriceType f = 0.0;
	f = pd->SettlementPrice < 999999.00 ? pd->SettlementPrice : pd->AveragePrice;
	resp.append(",\"settlementPrice\":");
	resp.append(to_string(f));
	resp.append(",\"upperLimitPrice\":");
	resp.append(to_string(pd->UpperLimitPrice));
	resp.append(",\"lowerLimitPrice\":");
	resp.append(to_string(pd->LowerLimitPrice));
	resp.append(",\"updateMillisec\":");
	resp.append(to_string(pd->UpdateMillisec));
	resp.append(",\"bidPrice1\":");
	resp.append(to_string(pd->BidPrice1));
	resp.append(",\"bidVolume1\":");
	resp.append(to_string(pd->BidVolume1));
	resp.append(",\"askPrice1\":");
	resp.append(to_string(pd->AskPrice1));
	resp.append(",\"askVolume1\":");
	resp.append(to_string(pd->AskVolume1));
	resp.append(",\"instrumentID\":\"");
	resp.append(pd->InstrumentID);
	resp.append("\",\"lastPrice\":");
	resp.append(to_string(pd->LastPrice));

	f = pd->PreSettlementPrice > 999999.0 ? 0 : pd->PreSettlementPrice;
	resp.append(",\"preSettlementPrice\":");
	resp.append(to_string(f));

	f = pd->PreClosePrice > 999999.0 ? 0 : pd->PreClosePrice;
	resp.append(",\"preClosePrice\":");
	resp.append(to_string(f));

	f = pd->PreOpenInterest > 999999.0 ? 0 : pd->PreOpenInterest;
	resp.append(",\"preOpenInterest\":");
	resp.append(to_string(f));

	f = pd->OpenPrice > 999999.0 ? pd->LastPrice : pd->OpenPrice;
	resp.append(",\"openPrice\":");
	resp.append(to_string(f));
	resp.append(",\"volume\":");
	resp.append(to_string(pd->Volume));

	resp.append(",\"turnover\":");
	resp.append(to_string(pd->Turnover));

	f = pd->OpenInterest > 999999.0 ? 0 : pd->OpenInterest;
	resp.append(",\"openInterest\":");
	resp.append(to_string(f));

	f = pd->ClosePrice > 999999.0 ? pd->LastPrice : pd->ClosePrice;
	resp.append(",\"closePrice\":");
	resp.append(to_string(f));
	resp.append(",\"updateTime\":\"");
	resp.append(pd->UpdateTime);
	resp.append("\",\"averagePrice\":");
	resp.append(to_string(pd->AveragePrice));
	resp.append("}");
	return true;
}

string vect2string(vector<string> &v)
{
	string s;
	for (auto x : v)
	{
		if (s.length() > 0)
		{
			s = s + "," + x;
		}
		else
		{
			s = x;
		}
	}
	return s;
}

void sync_to_monitor(int sockfd, const char *name, const char *src, int len)
{
	// 设置地址与端口
	struct sockaddr_in addr;
	socklen_t addr_len = sizeof(addr);
	memset(&addr, 0, sizeof(addr));
	addr.sin_family = AF_INET;

	for (auto &x : g_Config.sync[name])
	{
		addr.sin_port = htons(x.Port);
		addr.sin_addr.s_addr = inet_addr(x.Ips.c_str()); 
		sendto(sockfd, src, len, 0, (sockaddr *)&addr, addr_len);
	}
	//	close(sockfd);
}

bool file_exist(const char *filename){
        struct stat buffer;
        return stat(filename, &buffer) == 0;
}

int main(int argc, char *argv[])
{
	string defaultcfg = argv[0];
	defaultcfg.append(".txt");
	char *cfg;
	if (argc < 2 && !file_exist( defaultcfg.c_str() ))
        {
                print_usage();
                return 0;
        }
        if (argc > 1)
        {
                cfg = argv[1];
        }
        else
        {
                cfg = (char *)defaultcfg.c_str();
        }

	map<string, string> mconfig;
	readconfig(cfg, mconfig);
	for (auto &x : mconfig)
	{
		bool b = build_bunny((string &)x.first, (string &)x.second, g_Config);
		if (!b)
		{
			cout << "CAN'T PROCESS " << x.first << endl;
		}
	}
	if ( g_Config.m_front.size() <= 0) // || g_Config.subscribe.size() <= 0)
	{
		cout << "ERROR no define" << endl;
	}
	else
	{
		cout << g_Config << endl;
	}

	// instrument_count = g_Config.subscribe.size();
	instrument_count = g_Config.sync.size();
	instruments = (char **)malloc(sizeof(char *) * instrument_count);
	char **instrument = instruments;
	std::cout << "version:" << CThostFtdcMdApi::GetApiVersion() << std::endl;
	cout << "尝试订阅";
	for (auto iter= g_Config.sync.begin(); iter != g_Config.sync.end(); iter++)
	{
		*instrument = (char *) iter->first.c_str(); // InstA.c_str(
		cout << *instrument << ",";
		*instrument++;
	}
	/*
	for (int i = 0; i < g_Config.subscribe.size(); i++)
	{
		*instrument = (char *)g_Config.subscribe[i].c_str(); // InstA.c_str(
		cout << *instrument << ",";
		*instrument++;
		// *instrument = (char *)InstB.c_str();
	}
	*/
	cout << " 的行情信息" << endl;

	CThostFtdcMdApi *pApi = CThostFtdcMdApi::CreateFtdcMdApi("market");
	CMarketSpi Spi(pApi);
	for (auto &x : g_Config.m_front)
	{
		cout << "注册服务器：" << x << endl;
		pApi->RegisterFront((char *)x.c_str());
	}
	pApi->Init();
	////////////////////////////////////////////
	/*
	crow::SimpleApp app;
	CROW_ROUTE(app, "/")
	([]()
	 {
		 crow::json::wvalue x;
		 x["type"] = g_Config.type;
		 x["m_front"] = vect2string(g_Config.m_front);
		 x["subscribe"] = vect2string(g_Config.subscribe);
		 x["notify"] = vect2string(g_Config.notify);
		 x["user"] = g_Config.user;
		 x["brokerid"] = g_Config.brokerid;
		 x["authcode"] = g_Config.authcode;
		 x["appid"] = g_Config.appid;
		 x["t_front"] = vect2string(g_Config.t_front);
		 return x; });
	CROW_ROUTE(app, "/ctp/<string>")
	([](string name)
	 {
		 std::ostringstream os;
		 os << "if you want "<< name << ", please add it in subscribe!";
		 return crow::response(os.str()); });

	CROW_ROUTE(app, "/hello/<int>")
	([](int count)
	 {
		 if (count > 100)
		 return crow::response(400);
		 std::ostringstream os;
		 os << count << " bottles of beer!";
		 return crow::response(os.str()); });

	CROW_ROUTE(app, "/add_json")
		.methods("POST"_method)([](const crow::request &req)
								{
		 auto x = crow::json::load(req.body);
		 if (!x)
		 return crow::response(400);
		 int sum = x["a"].i()+x["b"].i();
		 std::ostringstream os;
		 os << sum;
		 return crow::response{os.str()}; });
	// app.loglevel(crow::LogLevel::Warning)
	int port = 9090;
	if (g_Config.port > 0)
	{
		port = g_Config.port;
	}
	app.loglevel(crow::LogLevel::Warning);
	app.port( port ).run();
	//app.port(port).multithreaded().run();
	// app will hung until CTRL+C
	//	std::this_thread::sleep_for(std::chrono::seconds(10));
	*/
	while(true){
		std::this_thread::sleep_for(std::chrono::seconds(10));
	}
	std::cout << "App be closed." << std::endl;
	return 0;
}
