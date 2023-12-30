#include <iostream>
#include <chrono>
#include <thread>
#include <time.h>
#include <algorithm>
#include <sstream>
#include <SFML/Graphics.hpp>
#include <sys/select.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <cstdlib>
#include <cstdio>
#include <sys/stat.h>
#include <numeric>

#ifdef _WIN32
#include "win/getopt.h"
#else
#include <unistd.h>
#endif
#include <cstring>
#include "httpclient.h"
#include "config.h"
#include "libs.h"
// #include "strategy.h"
#include "api/ThostFtdcUserApiStruct.h"

// 默认最大数据量，如果超过，降被丢弃。目前9:00～15:00，总计有效时间的10秒bar有 1350 个数据。
#define MAXLEN 1400
using namespace std;
using namespace std::chrono;

// 用于UDP接收数据buffer
char recv_buffer[1024];
// auto reqtime = std::chrono::steady_clock::now();
// 配置信息
Bunny g_Config;
// string vect2string(vector<string> &v);
struct TicksData
{
	double AveragePrice;
	string InstrumentID;
	double LastPrice;
	double OpenPrice;
	string TradingDay;
	string UpdateTime;
	int Volume;
};

struct SimpleMd
{
	double LastPrice;
	double AvgPrice;
	int Volume;
	float Radius;
};

struct MACDPoint
{
	double Price;
	double DiffMACD;
	double DiffX;
	string Time;
};

struct MouseClick
{
	float x;
	float y;
	double price;
	sf::Mouse::Button which;
	std::chrono::system_clock::time_point ctime;
};
/**
 * 特别提醒，windows update循环中，不要调用任何高消耗函数，如ta-lib调用，否则会导致程序崩溃。
 * 应该放入低频事件中（如接到远程数据更新后）调用。
 *
 */
void text_init(sf::Font &font);
void data_reset();
void save_data(string get_year_of_day);
void load_data(string year_of_day);
string get_year_of_day();
string get_file_name(const char *futuresId, string year_of_day);
bool load_data_from_bunny(string year_of_day);
void build_data_only_when_new_input(bool only_last = true);
void build_volume_vector(bool only_last = true);
void talib_RSI(int size, int period);
void draw_price_lines(sf::RenderWindow &win);
void draw_board(sf::RenderWindow &win);
void draw_decision_bar(sf::RenderWindow &win);
// bool volume_big_bang();
void indicator_macd(); // int slen, int blen);
void indicator_volume(sf::RenderWindow &win);
void update_last_data();
bool run_volume_strategy();
bool run_macd_strategy(); // double v1, double v2);
void run_analyse();

void print_volume();
void print_help();
void print_strategy();
// 当前系统采用的字体资源
sf::Font g_Font;
// 旗帜图片
sf::Texture g_FlagImg;
// 小旗帜图片的新增实例
vector<sf::Sprite> g_Flags;

// MACD窗口的差值序列，3角定位预测最高点。
vector<MACDPoint> g_MACDPoint;
// 用于判断是否双击， 和时间间隔组合使用
MouseClick g_LastClick;
// error counter
int g_ErrorNo;
// string F_Name = "AP401";
double g_OpenPrice;
double g_RSI;
// 数据执行日期，来自tick数据
string g_Datetime;
// vector<SimpleMd> g_Fdata;
std::map<string, SimpleMd> g_Fdata;
// 价格曲线
sf::Vertex g_Lines[MAXLEN + 1];
// Average LINEs
sf::Vertex g_avgLines[MAXLEN + 1];

// 主标题
sf::Text s_Main_Title;
// 当前价格（右侧）
sf::Text s_Price;
// 开始时间（左下方）
sf::Text s_Starttime;
// 当前日期（右侧）
sf::Text s_Datetime;
// 左上角MACD 红色标题
sf::Text s_Macd1;
// 左上角MACD 绿色标题
sf::Text s_Macd2;
// 底部指标
// sf::Text s_Indicator;
// MACD EMA（12）走势线
sf::Vertex g_xLines[MAXLEN + 1];
// MACD EMA（26）走势线
sf::Vertex g_yLines[MAXLEN + 1];
// 腰部 MACD  EMA（9）- EMA（26）的警示线
sf::Vertex g_xyLines[MAXLEN + 1];
// EMA（26）线的长度
int g_yLines_size;
// EMA（12）线的长度
int g_xLines_size;
// 警示圈
vector<sf::CircleShape> g_Warnning;
// 最大价格
double g_MaxPrice;
// 最小价格
double g_MinPrice;
// 最大交易量
double g_MaxVolume;
// 最小交易量
double g_MinVolume;
// 监控价格对比目标， < 0 表示非法目标；
double g_DiffPrice;
// 加速状态，当交易量突然异常（超过之前10分钟内最大值）增长的状态
bool g_Accelerate;
// 成交量
vector<int> g_Volume;

// 交易量-时间 TOPN
TopVolume g_TopV(50, 10);
Strategy g_Strategy(3);
AVStrategy g_AVStrategy(10.0);

// 价格数据
TA_Real g_InPrice[MAXLEN + 2];
// ta-lib计算结果数据buffer1
TA_Real g_Out1[MAXLEN + 2];
// ta-lib计算结果数据buffer2
TA_Real g_Out2[MAXLEN + 2];

// ta-lib计算Volume结果数据buffer1
// TA_Real g_VolOut1[MAXLEN + 1];
// ta-lib计算Volume结果数据buffer2
// TA_Real g_VolOut2[MAXLEN + 1];

void print_usage()
{
	std::cout << "example:\bunde config.txt 20231118\n 参数可选(optional), 默认为 config.txt 当日时间" << std::endl;
}

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
/**
 * 读取配置文件信息；根据配置，从远程服务端/本地读取数据；
 * 构造SFML窗口，并绘制数据；在事件监听中：
 * 1. 接收来自UDP的tick数据，并更新g_Fdata，并调用相关数据运算处理逻辑；
 * 2. 监听鼠标事件：左键双击，打印当前列对应数据；左右点击，增加小红旗；右键双击；删除所在位置小红旗；
 * 3. 监听键盘事件：F1：打印所有bar Volume数据；Delete：删除所有数据； Space：唤起分析程序，打印当前TOP情况；
 * 4. 窗口刷新（60fps，所以不要放入大运算逻辑，只专注绘制数据）
 *
 * 在tick数据更新逻辑中：
 * 1. g_Fdata已有数据更新，只更新对应line的最有一个点的信息，不做数据重新计算；以提高整体性能；
 * 2. g_Fdata新增数据，重新计算数据（MACD，Volume，RSI），并调用其他相关计算处理逻辑；
 */
int main(int argc, char *argv[])
{
	char *cfg;
	if (argc < 2 && !file_exist("./config.txt"))
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
		cfg = (char *)"./config.txt";
	}
	// default to set today as 20231118
	string year_of_day = get_year_of_day();
	if (argc > 2)
	{
		year_of_day = argv[2];
	}

	bool first_start = true;
	TA_RetCode retCode;
	retCode = TA_Initialize();
	if (retCode != TA_SUCCESS)
	{
		cerr << "Cannot initialize TA-Lib !" << retCode << endl;
		return -1;
	}
	std::cout << "Max Ticks Size = " << MAXLEN << endl;
	map<string, string> mconfig;
	readconfig(cfg, mconfig);
	for (auto &x : mconfig)
	{
		bool b = build_bunny((string &)x.first, (string &)x.second, g_Config);
		if (!b)
		{
			std::cout << "CAN'T PROCESS " << x.first << endl;
		}
	}
	if (g_Config.barsize <= 0.0)
		g_Config.barsize = 1.0;
	if (g_Config.volscale < 2.0)
		g_Config.volscale = 10.0;

	g_DiffPrice = -0.1;
	// 计算一天需要的bar数据长度。
	int dsize = tdiff("09:00:00", "15:00:00") / 10;
	g_Config.width = dsize * g_Config.barsize + 2 * BORDER_WIDTH + RIGHT_WTH;
	g_Config.bufsize = dsize;
	if (dsize > MAXLEN)
	{
		g_Config.width = MAXLEN * g_Config.barsize + 2 * BORDER_WIDTH + RIGHT_WTH;
		g_Config.bufsize = MAXLEN;
	}
	std::cout << "tdiff() = " << dsize << endl;

	if (g_Config.high < 600)
		g_Config.high = WIN_HIGH;

	std::cout << g_Config << endl;
	int sockfd;
	// 创建socket
	sockfd = socket(AF_INET, SOCK_DGRAM, 0);
	if (-1 == sockfd)
	{
		return 0;
		puts("Failed to create socket");
	}

	// 设置地址与端口
	struct sockaddr_in addr;
	socklen_t addr_len = sizeof(addr);
	memset(&addr, 0, sizeof(addr));
	addr.sin_family = AF_INET;			  // Use IPV4
	addr.sin_port = htons(g_Config.port); //
	addr.sin_addr.s_addr = htonl(INADDR_ANY);
	// Time out
	struct timeval tv;
	tv.tv_sec = 0;
	tv.tv_usec = 20000; // 20 ms, 太长容易阻碍刷新流畅度
	setsockopt(sockfd, SOL_SOCKET, SO_RCVTIMEO, (const char *)&tv, sizeof(struct timeval));
	// Bind 端口，用来接受之前设定的地址与端口发来的信息,作为接受一方必须bind端口，并且端口号与发送方一致
	if (::bind(sockfd, (struct sockaddr *)&addr, addr_len) == -1)
	{
		printf("Failed to bind socket on port %d\n", g_Config.port);
		close(sockfd);
		return 0;
	}
	struct sockaddr_in src;
	socklen_t src_len = sizeof(src);
	// 加载flag图片
	g_FlagImg.loadFromFile(g_Config.fFlag.c_str()); //  "./flag.jpeg"
	////////////////////////////////////////////
	sf::RenderWindow window(sf::VideoMode(g_Config.width, g_Config.high), "Bunde, customized futures monitor @lengss");
	// 加载字体
	if (!g_Font.loadFromFile(FONT_FILE))
	{
		std::cout << "Failed to load font." << std::endl;
		return -1;
	}
	// text title init
	text_init(g_Font);
	// 加载数据（根据配置，从远程服务端/本地读取数据）
	load_data(year_of_day);
	// Framerate
	window.setFramerateLimit(60);
	while (window.isOpen())
	{
		sf::Event event;
		while (window.pollEvent(event))
		{
			if (event.type == sf::Event::Closed)
			{
				save_data(year_of_day);
				window.close();
			}
			if (event.type == sf::Event::KeyPressed)
			{
				// q exit
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::Q))
				{
					save_data(year_of_day);
					window.close();
				}
				// Escape取消盯位
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::Escape))
				{
					cout << " ***** 取消盯位 ****** " << endl;
					g_DiffPrice = 0.0;
				}
				// 空格键可以寻求帮助
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::Space))
					run_analyse();
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::F1))
					print_help();
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::F2))
					print_strategy();
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::F3))
					print_volume();
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::Delete))
					data_reset();
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::Add))
				{
					cout << " ***** 盯涨势位 ****** " << endl;
					if (g_LastClick.price <= g_MaxPrice && g_LastClick.price >= g_MinPrice)
					{
						g_DiffPrice = g_LastClick.price;
						string st = "unkown";
						int idx = (g_LastClick.x - BORDER_WIDTH) / g_Config.barsize;
						if (idx < g_Fdata.size())
						{
							auto it = g_Fdata.begin();
							for (; idx > 0; idx--)
								it++;
							st = it->first;

							if (g_Strategy.Buy(g_DiffPrice, st))
								it->second.Radius = -10.0;

							cout << "#设置新的对比点位：" << st << "," << to_string(g_DiffPrice) << endl;
						}
					}
					else
					{
						cout << " ***** 请先鼠标左键双击希望对比的价格位置，然后按'+' ****** " << endl;
					}
				}
				if (sf::Keyboard::isKeyPressed(sf::Keyboard::Subtract))
				{
					cout << " ***** 盯跌势位 ****** " << endl;
					if (g_LastClick.price <= g_MaxPrice && g_LastClick.price >= g_MinPrice)
					{
						g_DiffPrice = -1 * g_LastClick.price;
						string st = "unkown";
						int idx = (g_LastClick.x - BORDER_WIDTH) / g_Config.barsize;
						if (idx < g_Fdata.size())
						{
							auto it = g_Fdata.begin();
							for (; idx > 0; idx--)
								it++;
							st = it->first;

							if (g_Strategy.Sell(g_LastClick.price, st))
								it->second.Radius = -10.0;

							cout << "#设置新的对比点位：" << st << ", " << to_string(g_DiffPrice) << endl;
						}
					}
					else
					{
						cout << " ***** 请先鼠标左键双击希望对比的价格位置，然后按'-' ****** " << endl;
					}
				}
			}
			// 左右键控制增加的图片，左+右：添加； 双右键：删除
			if (event.type == sf::Event::MouseButtonPressed)
			{
				if (event.mouseButton.button == sf::Mouse::Left)
				{
					sf::Vector2i pos = sf::Mouse::getPosition(window);
					sf::Vector2f posf = window.mapPixelToCoords(pos);
					if (posf.x > 0 && posf.x < g_Config.width && posf.y > 0 && posf.y < g_Config.high)
					{

						auto rsptime = std::chrono::system_clock::now();
						if (posf.x == g_LastClick.x && posf.y == g_LastClick.y)
						{
							chrono::seconds sec = chrono::duration_cast<chrono::seconds>(rsptime - g_LastClick.ctime);
							if (sec.count() < 5 && g_LastClick.which == sf::Mouse::Left) // 5s内双击
							{
								cout << "-----------  tips for you------------\n";
								if (g_Fdata.size() > 0)
								{
									auto it = g_Fdata.end();
									it--;
									cout << "new @" << it->first << " : " << it->second.LastPrice << "\n";
									int idx = int(posf.x - BORDER_WIDTH) / g_Config.barsize;
									if (idx >= 0 && idx < g_Fdata.size())
									{
										auto it2 = g_Fdata.begin();
										for (; idx > 0; idx--)
											it2++;
										cout << "pos @" << it2->first << " : " << it2->second.LastPrice << "; 与当前差价 = " << it2->second.LastPrice - it->second.LastPrice << "; Avg = " << it2->second.AvgPrice << "\n";
										g_LastClick.price = it2->second.LastPrice;
									}
									cout << endl;
								}
							}
						}
						g_LastClick.ctime = rsptime;
						g_LastClick.x = posf.x;
						g_LastClick.y = posf.y;
						g_LastClick.which = sf::Mouse::Left;
					}
				}
				if (event.mouseButton.button == sf::Mouse::Right)
				{
					sf::Vector2i pos = sf::Mouse::getPosition(window);
					sf::Vector2f posf = window.mapPixelToCoords(pos);
					if (posf.x > 0 && posf.x < g_Config.width && posf.y > 0 && posf.y < g_Config.high)
					{
						auto rsptime = std::chrono::system_clock::now();
						if (posf.x == g_LastClick.x && posf.y == g_LastClick.y)
						{

							chrono::seconds sec = chrono::duration_cast<chrono::seconds>(rsptime - g_LastClick.ctime);
							if (sec.count() < 5 && g_LastClick.which == sf::Mouse::Left)
							{
								sf::Sprite flag(g_FlagImg);
								flag.setPosition(posf.x - 25, posf.y - 30); // 25, 30 是针对旗杆的位置矫正，
								flag.setScale({0.2, 0.2});
								g_Flags.push_back(flag);
							}
							int ct = g_Flags.size();
							if (sec.count() < 5 && g_LastClick.which == sf::Mouse::Right && ct > 0)
							{
								for (int i = ct - 1; i >= 0; i--)
								{
									float x = g_Flags[i].getPosition().x;
									float y = g_Flags[i].getPosition().y;
									// 之前使用了 size_t i = 0; 导致i永远是>=0!! 如果没有下面的逻辑，将无限循环并导致crash
									// if (x<= 0 || y<= 0) break;
									// 保留这行代码，作为以后警示和学习！

									if (posf.x < x + 12.5 + 30 && posf.x > x + 12.5 - 30 && posf.y < y + 15 + 30 && posf.y > y + 15 - 30)
									{
										g_Flags.erase(g_Flags.begin() + i);
										break;
									}
								}
							}
						}
						g_LastClick.ctime = rsptime;
						g_LastClick.x = posf.x;
						g_LastClick.y = posf.y;
						g_LastClick.which = sf::Mouse::Right;
					}
				}
			}
		}
		// UDP server
		memset(&src, 0, sizeof(src));
		// 阻塞20ms, receive消息
		int sz = recvfrom(sockfd, recv_buffer, 1024, 0, (sockaddr *)&src, &src_len);
		if (sz > 0 && sz < 1024)
		{
			recv_buffer[sz] = 0;
		//	CThostFtdcDepthMarketDataField t = bufToStruct<CThostFtdcDepthMarketDataField>((void *)recv_buffer);
			vector<string> rec_v = stringSplit(recv_buffer,';');
			TicksData t;
			if (rec_v.size() == 9){
				t.AveragePrice = atof(rec_v[0].c_str());
				t.InstrumentID = rec_v[3] ;
				t.LastPrice = atof(rec_v[4].c_str());
				t.OpenPrice = atof(rec_v[5].c_str());
				t.TradingDay = rec_v[6];
				t.UpdateTime = rec_v[7];
				t.Volume = atoi(rec_v[8].c_str());
				if (g_Config.av_args !=0.0 && g_Config.av_args!=1.0){
					t.AveragePrice = t.AveragePrice/g_Config.av_args;
				}
			}

			string lastt = "";
			int lastvolume = 0;
			auto it = g_Fdata.end();
			if (g_Fdata.size() > 0)
			{
				//	auto it = g_Fdata.end();
				it--;
				lastt = it->first;
				lastvolume = it->second.Volume;
			}

			if (strcmp(t.UpdateTime.c_str(), lastt.c_str()) >= 0 && t.Volume > lastvolume)
			{
				if (g_OpenPrice == 0)
				{
					g_OpenPrice = t.OpenPrice;
				}
				if (strcmp(g_Datetime.c_str(), t.TradingDay.c_str()) != 0)
				{
					g_Datetime = t.TradingDay;
				}
				double minp = min(t.AveragePrice, t.LastPrice);
				if (minp == 0)
				{
					// cout << "average = 0, error data source !" << endl;
					minp = (int(t.LastPrice) / 10) * 10;
					t.AveragePrice = minp;
				}
				if (g_MinPrice == 0.0)
				{
					g_MinPrice = minp - 2;
				}
				else if (g_MinPrice > minp)
					g_MinPrice = minp - 2;

				double maxp = max(t.AveragePrice, t.LastPrice);
				if (g_MaxPrice < maxp)
					g_MaxPrice = maxp + 2;

				// 对UpdateTime 进行处理，抹掉个位秒数，用0代替，实现以10秒为单位保存数据；
				string stime_key = t.UpdateTime; // 10:10:10
				if (!g_Config.seconds)
					stime_key = stime_key.substr(0, 7) + "0";

				if (it != g_Fdata.end() && it->first == stime_key)
				{
					// 计算平均价, 非线形平均，最近的价格影响最大。
					double lastv = it->second.LastPrice;
					it->second.LastPrice = (t.LastPrice + lastv) / 2;
					it->second.Volume = t.Volume;
					// 只更新最后一个价格
					update_last_data();
				}
				else if (it == g_Fdata.end() || stime_key.compare(it->first) > 0)
				{
					SimpleMd sm = SimpleMd{t.LastPrice, t.AveragePrice, t.Volume, 0.0};
					g_Fdata.insert(std::pair<string, SimpleMd>(stime_key, sm));

					if (g_MACDPoint.size() == 0)
					{
						MACDPoint x = MACDPoint{t.LastPrice, 0, 0, stime_key};
						g_MACDPoint.push_back(x);
					}
					// 只有数据增加时才进行计算
					// big_bang();
					// volume_big_bang();
					build_data_only_when_new_input(true);
				}

				first_start = false;
			}
			else // 根据时间判断，如果不是新数据，则认为有问题，提醒重置（Delete一键重置，或新启动指定数据源）
			{
				if (g_ErrorNo == 0)
				{
					cout << "error data:" << t.UpdateTime << " < " << lastt << "; 可以DEL清除旧数据！" << endl;
				}
				g_ErrorNo = (g_ErrorNo + 1) % (1000);
			}
		}
		else if (first_start) // 第一次启动时，从文件载入数据，需要进行计算处理
		{
			// 对读入的数据做一次性构造计算处理。
			build_data_only_when_new_input(false);
			first_start = false;
		}

		window.clear();
		int size = g_Fdata.size();
		if (size > 6) // 1分钟的数据
		{
			s_Main_Title.setString(g_Config.fName + ":" + g_Datetime);
			window.draw(s_Main_Title);
			// 主价格曲线
			draw_price_lines(window);
			// 4色决策bar
			draw_decision_bar(window);
			// 隔离线
			sf::Vertex _lines[] = {
				sf::Vertex(sf::Vector2f(10, g_Config.high - WIN_INDICATOR + 50), sf::Color(255, 200, 200)),
				sf::Vertex(sf::Vector2f(g_Config.width - BORDER_WIDTH, g_Config.high - WIN_INDICATOR + 50), sf::Color(255, 200, 200))};
			window.draw(_lines, 2, sf::Lines);
			// 底部volume指标线
			indicator_volume(window);
		}
		else
		{ // 显示 无信号 。。。
			sf::Text info;
			info.setFont(g_Font);
			info.setPosition({float(g_Config.width) / 3, float((g_Config.high - WIN_INDICATOR) / 2)});
			info.setString("No signal, or No data, Please wait ...");
			info.setFillColor(sf::Color::Red);
			info.setCharacterSize(46);
			window.draw(info);
		}
		if (g_Flags.size() > 0)
		{
			for (auto x : g_Flags)
			{
				window.draw(x);
			}
		}
		window.display();
	}
	// app will hung until CTRL+C
	close(sockfd);
	//	std::this_thread::sleep_for(std::chrono::seconds(10));
	TA_Shutdown();
	std::cout << "App be closed." << std::endl;
	return 0;
}

void data_reset()
{
	g_Strategy.Clear();
	g_AVStrategy.Clear();
	g_TopV.clear();
	g_Volume.clear();
	g_Fdata.clear();
	g_xLines_size = 0;
	g_yLines_size = 0;
	g_Warnning.clear();
	g_Flags.clear();
	g_MaxPrice = 0;
	g_MinPrice = 0;
	g_MaxVolume = 0;
	g_MinVolume = 0;
	g_Accelerate = false;
	g_Datetime.clear();
	g_OpenPrice = 0;
	g_MACDPoint.clear();
	g_DiffPrice = 0.0;
	mylog("WARNING", "CLEAR ALL DATA!!", 31, 43, 5);
	cout << "... ... ......" << endl;
}
/*
价格走势曲线，包括相应的Title信息。
*/
void draw_price_lines(sf::RenderWindow &win)
{
	auto last = g_Fdata.end();
	last--;

	s_Starttime.setString(g_Fdata.begin()->first);
	int size = g_Fdata.size();
	s_Datetime.setString("UpdateTime: " + last->first);
	char s[32];
	snprintf(s, 32, "LastPrice: %.2f", last->second.LastPrice);
	s_Price.setString(s);
	win.draw(s_Starttime);
	win.draw(s_Datetime);
	win.draw(s_Price);
	win.draw(s_Macd1);
	win.draw(s_Macd2);

	draw_board(win);
	win.draw(g_Lines, size, sf::PrimitiveType::LineStrip);
	win.draw(g_avgLines, size, sf::PrimitiveType::LineStrip);
	// MACD的0轴线
	sf::Vertex _lines[] = {
		sf::Vertex(sf::Vector2f(BORDER_WIDTH, g_Config.high - WIN_INDICATOR), sf::Color(128, 128, 128)),
		sf::Vertex(sf::Vector2f(g_Config.width - BORDER_WIDTH, g_Config.high - WIN_INDICATOR), sf::Color(128, 128, 128))};
	win.draw(_lines, 2, sf::Lines);

	// EMA(12)，MEA(26)的走势线，以及差值警示线
	if (g_xLines_size > 2)
	{
		win.draw(g_xLines, g_xLines_size, sf::PrimitiveType::LineStrip);
	}

	if (g_yLines_size > 2)
	{
		win.draw(g_yLines, g_yLines_size, sf::PrimitiveType::LineStrip);
		win.draw(g_xyLines, g_yLines_size, sf::PrimitiveType::LineStrip);
	}
	float last_x = 0.0;

	// Warning圈和Warning线
	for (int i = 0; i < g_Warnning.size(); i++)
	{
		win.draw(g_Warnning[i]);

		float xx = g_Warnning[i].getPosition().x + g_Warnning[i].getRadius();
		if (xx - last_x < g_Config.barsize * 3)
		{
			continue;
		}
		float yy = g_Warnning[i].getPosition().y;
		last_x = xx;

		sf::Vertex _lines[] = {
			sf::Vertex(sf::Vector2f(xx, yy), sf::Color(255, 50, 0, 100)),
			sf::Vertex(sf::Vector2f(xx, 100), sf::Color(255, 50, 0, 100))};
		win.draw(_lines, 2, sf::Lines);
	}

	//
	//  成交点 for strategy
	//
	int startx = BORDER_WIDTH;
	int starty = g_Config.high - WIN_INDICATOR - 40;
	int high = g_Config.high - WIN_INDICATOR - 100;

	if ((g_MaxPrice - g_MinPrice) == 0.0)
		return;
	double step_y = (1.0 * high) / (g_MaxPrice - g_MinPrice);

	int i = 0;
	int idx = 0;

	for (map<string, SimpleMd>::iterator it = g_Fdata.begin(); it != g_Fdata.end(); ++it, i++)
	{
		float r = it->second.Radius;
		if (r < 0.0)
		{
			sf::CircleShape c(r * (-1));
			float x = startx + i * g_Config.barsize;
			float y = starty - step_y * (it->second.LastPrice - g_MinPrice);
			c.setPosition(x + r, y + r); // 此时 r 为负数
			// kai = RED, ping = BLUE
			sf::Color color = idx % 2 == 0 ? sf::Color(255, 0, 0, 168) : sf::Color(0, 0, 255, 168);
			c.setFillColor(color);
			win.draw(c);
			idx++;
		}
	}
	// 画策略分界线
	// float step_y = (1.0 * high) / (g_MaxPrice - g_MinPrice);
	// int size = g_Fdata.size();

	auto iter = g_Fdata.begin();
	float x0 = startx;
	// +/- 1
	double v1a = iter->second.AvgPrice + g_Config.dvalue;
	double v1b = iter->second.AvgPrice - g_Config.dvalue;
	// +/- 2
	double v2a = iter->second.AvgPrice + 2 * g_Config.dvalue;
	double v2b = iter->second.AvgPrice - 2 * g_Config.dvalue;
	// +/- 4
	double v4a = iter->second.AvgPrice + 4 * g_Config.dvalue;
	double v4b = iter->second.AvgPrice - 4 * g_Config.dvalue;

	iter++;
	for (i = 1; iter != g_Fdata.end(); ++iter, i++)
	{
		// +/-  1*dvalue
		double v1a2 = iter->second.AvgPrice + g_Config.dvalue;
		double v1b2 = iter->second.AvgPrice - g_Config.dvalue;

		float x2 = startx + i * g_Config.barsize;	
		bool resetx = false;	
		if ( v1a < g_MaxPrice && v1a2 < g_MaxPrice )
		{			
			float y1 = (float)(starty - (v1a - g_MinPrice) * step_y);
			float y2 = (float)(starty - (v1a2 - g_MinPrice) * step_y);

			sf::Vertex _lines[] = {
				sf::Vertex(sf::Vector2f(x0, y1), sf::Color(255, 0, 0, 120)),
				sf::Vertex(sf::Vector2f(x2, y2), sf::Color(255, 0, 0, 120))};
			win.draw(_lines, 2, sf::Lines);
			v1a = v1a2;
			resetx = true;
		}

		if ( v1b > g_MinPrice && v1b2 > g_MinPrice )
		{
			float y1 = (float)(starty - ( v1b - g_MinPrice ) * step_y);
			float y2 = (float)(starty - ( v1b2 - g_MinPrice ) * step_y);

			sf::Vertex _lines[] = {
				sf::Vertex(sf::Vector2f(x0, y1), sf::Color(255, 0, 0, 120)),
				sf::Vertex(sf::Vector2f(x2, y2), sf::Color(255, 0, 0, 120))};
			win.draw(_lines, 2, sf::Lines);
			v1b = v1b2;
			resetx = true;
		}
		
		// +/- 2*dvalue    粉色：255, 0 , 255
		double v2a2 = iter->second.AvgPrice + 2*g_Config.dvalue;
		double v2b2 = iter->second.AvgPrice - 2*g_Config.dvalue;

		if ( v2a < g_MaxPrice && v2a2 < g_MaxPrice )
		{			

			float y1 = (float)(starty - (v2a - g_MinPrice) * step_y);
			float y2 = (float)(starty - (v2a2 - g_MinPrice) * step_y);

			sf::Vertex _lines[] = {
				sf::Vertex(sf::Vector2f(x0, y1), sf::Color(255, 0, 255, 120)),
				sf::Vertex(sf::Vector2f(x2, y2), sf::Color(255, 0, 255, 120))};
			win.draw(_lines, 2, sf::Lines);
			v2a = v2a2;
			resetx = true;
		}

		if ( v2b > g_MinPrice && v2b2 > g_MinPrice )
		{
		
			float y1 = (float)(starty - ( v2b - g_MinPrice ) * step_y);
			float y2 = (float)(starty - ( v2b2 - g_MinPrice ) * step_y);

			sf::Vertex _lines[] = {
				sf::Vertex(sf::Vector2f(x0, y1), sf::Color(255, 0, 255, 120)),
				sf::Vertex(sf::Vector2f(x2, y2), sf::Color(255, 0, 255, 120))};
			win.draw(_lines, 2, sf::Lines);
			v2b = v2b2;
			resetx = true;
		}
	
		// +/- 4*dvalue    紫色：160 32 240
		double v4a2 = iter->second.AvgPrice + 4*g_Config.dvalue;
		double v4b2 = iter->second.AvgPrice - 4*g_Config.dvalue;

		if ( v4a < g_MaxPrice && v4a2 < g_MaxPrice )
		{			

			float y1 = (float)(starty - (v4a - g_MinPrice) * step_y);
			float y2 = (float)(starty - (v4a2 - g_MinPrice) * step_y);

			sf::Vertex _lines[] = {
				sf::Vertex(sf::Vector2f(x0, y1), sf::Color(160, 32, 240, 120)),
				sf::Vertex(sf::Vector2f(x2, y2), sf::Color(160, 32, 240, 120))};
			win.draw(_lines, 2, sf::Lines);
			v4a = v4a2;
			resetx = true;
		}

		if ( v4b > g_MinPrice && v4b2 > g_MinPrice )
		{
		
			float y1 = (float)(starty - ( v4b - g_MinPrice ) * step_y);
			float y2 = (float)(starty - ( v4b2 - g_MinPrice ) * step_y);

			sf::Vertex _lines[] = {
				sf::Vertex(sf::Vector2f(x0, y1), sf::Color(160, 32, 240, 120)),
				sf::Vertex(sf::Vector2f(x2, y2), sf::Color(160, 32, 240, 120))};
			win.draw(_lines, 2, sf::Lines);
			v4b = v4b2;
			resetx  = true;
		}
		if (resetx) x0 = x2;
	}
}
/**
 * @brief 构造价格走势相关的曲线数据，主要是构造MACD，RSI, Volume EMA的数据
 * 此函数太重，不能放在事件流程中按刷新频率调用，必须在更低频的控制中调用，如接收到新数据的流程中。 否则程序会crash
 *
 */
void build_data_only_when_new_input(bool only_last)
{
	int delc = 0;
	if (g_Fdata.size() > g_Config.bufsize)
	{
		cout << "try to resize g_Fdata ... ..." << endl;
		int count = g_Fdata.size();
		int save = g_Config.bufsize > 300 ? 60 : count / 5;
		auto it = g_Fdata.begin();
		auto next = it++;
		for (int i = 0; i < count - save; i++)
		{
			string key = it->first;
			if (key.length() == 8 && key.c_str()[7] != '0')
			{
				g_Fdata.erase(it);
				delc += 1;
			}
			it = next;
			next++;
		}
	}
	while (g_Fdata.size() > g_Config.bufsize)
	{
		g_Fdata.erase(g_Fdata.begin());
		delc += 1;
	}

	if (delc > 0)
	{
		only_last = false;
		g_TopV.clear();
		g_Volume.clear();
		g_xLines_size = 0;
		g_yLines_size = 0;
		g_Warnning.clear();
		g_Accelerate = false;
		g_MACDPoint.clear();
	}

	int size = g_Fdata.size();
	if (!only_last)
	{
		int i = 0;
		for (map<string, SimpleMd>::iterator it = g_Fdata.begin(); it != g_Fdata.end(); ++it, i++)
			g_InPrice[i] = it->second.LastPrice;
	}

	if (size > 0)
	{
		auto it = g_Fdata.end();
		it--;
		g_InPrice[size - 1] = it->second.LastPrice;
	}

	// int size = g_Fdata.size();

	TA_Integer outEma_12_start;
	TA_Integer outEma_12_len;
	TA_Integer outEma_26_start;
	TA_Integer outEma_26_len;

	if (size > MACD_EMA_BIG + 2)
	{ /*
		 int i = 0;
		 for (map<string, SimpleMd>::iterator it = g_Fdata.begin(); it != g_Fdata.end(); ++it)
		 {
			 g_InPrice[i] = it->second.LastPrice;
			 i += 1;
		 }
 */
		TA_RetCode retCode = TA_MA(0, size - 1, &g_InPrice[0], MACD_EMA_SMALL, TA_MAType_EMA, &outEma_12_start, &outEma_12_len, &g_Out1[0]);
		if (retCode != TA_SUCCESS)
		{
			return;
		}

		retCode = TA_MA(0, size - 1, &g_InPrice[0], MACD_EMA_BIG, TA_MAType_EMA, &outEma_26_start, &outEma_26_len, &g_Out2[0]);
		if (retCode != TA_SUCCESS)
		{
			return;
		}
	}

	int startx = BORDER_WIDTH;
	int starty = g_Config.high - WIN_INDICATOR - 40;
	int high = g_Config.high - WIN_INDICATOR - 100;
	int width = g_Config.width - RIGHT_WTH - 2 * BORDER_WIDTH;

	if ((g_MaxPrice - g_MinPrice) == 0.0)
		return;

	double step_y = (1.0 * high) / (g_MaxPrice - g_MinPrice);
	float step_x = g_Config.barsize;

	if (size > 30)
	{
		float left26 = step_x * (MACD_EMA_BIG - 1);	  //  (outEma_26_start - 1);
		float left12 = step_x * (MACD_EMA_SMALL - 1); //(outEma_12_start - 1);

		for (int i = 0; i < size - MACD_EMA_BIG /* outEma_26_len */; i++)
		{
			float x = startx + left26 + i * step_x;
			float y = (float)(starty - (g_Out2[i] - g_MinPrice) * step_y);
			g_yLines[i] = sf::Vertex(sf::Vector2f(x, y), sf::Color::Green);
		}

		for (int i = 0; i < size - MACD_EMA_SMALL /* outEma_12_len */; i++)
		{
			float x = startx + left12 + i * step_x;
			float y = (float)(starty - (g_Out1[i] - g_MinPrice) * step_y);
			g_xLines[i] = sf::Vertex(sf::Vector2f(x, y), sf::Color::Red);
		}

		g_yLines_size = size - MACD_EMA_BIG;   //  outEma_26_len;
		g_xLines_size = size - MACD_EMA_SMALL; // outEma_12_len;
		int start_y = g_Config.high - WIN_INDICATOR;
		// 构造macd涨跌指示线
		for (int i = 0; i < size - MACD_EMA_BIG /* outEma_26_len */; i++)
		{
			int move = MACD_EMA_BIG - MACD_EMA_SMALL; //   outEma_26_start - outEma_12_start;
			float x = startx + left26 + i * step_x;
			float yv = (float)(g_Out1[i + move] - g_Out2[i]);
			sf::Color c = yv > 0.0 ? sf::Color::Red : sf::Color(75, 200, 75);

			yv = yv * 10;
			if (yv > 28.0)
			{
				yv = 28.0;
				c = sf::Color(238, 130, 238);
			}
			else if (yv < -28.0)
			{
				yv = -28.0;
				c = sf::Color(0, 255, 0);
			}

			float y = (float)(start_y - yv);
			g_xyLines[i] = sf::Vertex(sf::Vector2f(x, y), c);
		}
		//
		// float x = startx + left26 + (outEma_26_len - 2) * step_x - 1;
		indicator_macd(); // outEma_12_len, outEma_26_len);
	}
	g_Warnning.clear();
	// 交易警示圈 ！！！
	int i = 0;
	for (map<string, SimpleMd>::iterator it = g_Fdata.begin(); it != g_Fdata.end(); ++it, i++)
	{
		float x = startx + i * step_x;
		float y = (float)(starty - (it->second.LastPrice - g_MinPrice) * step_y);
		g_Lines[i] = sf::Vertex(sf::Vector2f(x, y), sf::Color::White);

		float y2 = (float)(starty - (it->second.AvgPrice - g_MinPrice) * step_y);
		g_avgLines[i] = sf::Vertex(sf::Vector2f(x, y2), sf::Color::Yellow);

		if (it->second.Radius != 0.0)
		{
			float r = it->second.Radius;
			sf::Color cc = r > g_Config.dvalue ? sf::Color::Red : sf::Color::Green;
			if (r < 0.0)
				r = -1 * r;
			r = r > 10 ? 4 : r / 3;
			sf::CircleShape c(r);
			// zuo yi ban ge r
			c.setPosition(x - step_x - r, g_Config.high - WIN_INDICATOR - 30);
			c.setFillColor(cc);
			g_Warnning.push_back(c);
		}
	}
	// tabli_RSI中没有构造 g_InPrice数据，需要复用这里的数据！！
	// talib_RSI(size, RSI_PERIOD);
	// 根据g_Fdata构造g_Volume,便于其他函数直接使用g_Volume

	build_volume_vector(only_last);
}

/**
 * 更新价格走势曲线的最后一个点和最后一个Volume值。
 */
void update_last_data()
{
	int size = g_Fdata.size();
	if (size < 2)
		return;

	int startx = BORDER_WIDTH;
	int starty = g_Config.high - WIN_INDICATOR - 40;
	int high = g_Config.high - WIN_INDICATOR - 100;
	int width = g_Config.width - RIGHT_WTH - 2 * BORDER_WIDTH;

	if ((g_MaxPrice - g_MinPrice) == 0.0)
		return;

	double step_y = (1.0 * high) / (g_MaxPrice - g_MinPrice);
	float step_x = g_Config.barsize;

	int i = g_Fdata.size() - 1;
	auto it = g_Fdata.end();
	it--;

	float x = startx + i * step_x;
	float y = (float)(starty - (it->second.LastPrice - g_MinPrice) * step_y);
	sf::Color c = sf::Color::White;

	g_Lines[i] = sf::Vertex(sf::Vector2f(x, y), c);
	if (it->second.Radius > 0.1)
	{
		float r = it->second.Radius;
		sf::Color cc = r > g_Config.dvalue ? sf::Color::Red : sf::Color::Green;
		if (r < 0.0)
			r = -1 * r;
		r = r > 10 ? 4 : r / 3;
		sf::CircleShape c(r);
		// zuo yi ban ge r
		c.setPosition(x - step_x - r, g_Config.high - WIN_INDICATOR - 30);
		c.setFillColor(cc);
		g_Warnning.push_back(c);
	}
	// update last g_Volume
	int lastv = it->second.Volume;
	it--;
	if (g_Volume.size() == g_Fdata.size())
	{
		g_Volume[g_Volume.size() - 1] = lastv - it->second.Volume;
	}
	else
	{
		cout << " error to update g_Volume in update_xxx " << g_Volume.size() << " : " << g_Fdata.size() << endl;
		build_volume_vector(false);
	}
}

/**
 * 腰部的MACD 指标，涨势用红线，在上；跌势用绿线在下。
 * 此函数必须放在计算MACD之后紧跟着调用，以便于借用计算出来的数据。
 */
void indicator_macd() // int outEma_12_len, int outEma_26_len)
{
	//
	int size = g_Fdata.size();
	int outEma_12_len = size - MACD_EMA_SMALL;
	int outEma_26_len = size - MACD_EMA_BIG;

	if (outEma_26_len <= 3)
		return;
	int move = outEma_12_len - outEma_26_len;
	double v1 = g_Out1[outEma_12_len - 1] - g_Out2[outEma_26_len - 1];
	double v2 = g_Out1[outEma_12_len - 2] - g_Out2[outEma_26_len - 2];
	auto it = g_Fdata.end();
	it--;

	if (v1 >= 0 && v2 <= 0)
	{ // Up，金交叉，卖空，平多机会
		g_MACDPoint.clear();
	}
	else if (v1 <= 0 && v2 >= 0)
	{ // Down， 银交叉， 买多、平空机会。
		g_MACDPoint.clear();
	}
	else
	{ // MACD窗口，3角定位寻找最大差值。

		if (g_MACDPoint.size() == 0)
			return;
		double vv = v1;
		double diffx = 0; // it->second.LastPrice - g_Out2[outEma_26_len - 1];
		if (v1 > 0)
		{
			if (g_DiffPrice > 0) //+
				diffx = it->second.LastPrice - g_DiffPrice;
			else
				diffx = it->second.LastPrice - it->second.AvgPrice;
		}
		else
		{
			v1 = -1 * v1;
			if (g_DiffPrice < 0.0)
				diffx = (-1 * g_DiffPrice) - it->second.LastPrice;
			else
				diffx = it->second.AvgPrice - it->second.LastPrice;
		}

		MACDPoint x = MACDPoint{it->second.LastPrice, v1, diffx, it->first};
		g_MACDPoint.push_back(x);
		int xsize = g_MACDPoint.size();

		if (xsize > 5)
		{
			auto max_it = max_element(g_MACDPoint.begin(), g_MACDPoint.end(), [](MACDPoint a, MACDPoint b)
									  { return a.DiffMACD < b.DiffMACD; });
			if (g_MACDPoint[xsize - 2].DiffMACD == max_it->DiffMACD)
			{
				// run AVStrategy ...
				if (vv * (it->second.LastPrice - it->second.AvgPrice) > 0)
					g_AVStrategy.Next(it->second.LastPrice, it->second.AvgPrice, it->first);

				if (max_it->DiffMACD > 0.2 && diffx > g_Config.dvalue)
				{
					char s[102];
					snprintf(s, 102, "MACD拐点: Price = %.2f, DiffValue = %.2f", it->second.LastPrice, diffx);
					warning_log("MACD", s, 35, 47, 0);

					if (norepeat())
						macd_warnning_to_notify(g_Config.fName, max_it->DiffMACD, g_Config.notify);

					if (run_macd_strategy())
					{								  // max_it->DiffMACD, max_it->DiffX))
						if (it->second.Radius == 0.0) // 没有被策略实施占用
							it->second.Radius = diffx;
					}
				}
				g_MACDPoint[0].Price = max_it->Price;
				g_MACDPoint[0].Time = max_it->Time;
			}
		}
	}
}
/**
 * @brief 文本组件的初始化
 */
void text_init(sf::Font &font)
{
	s_Main_Title.setFont(font);
	s_Main_Title.setPosition({float(g_Config.width) / 2 - 30, 10});
	s_Main_Title.setString(g_Config.fName + ":" + g_Datetime);
	s_Main_Title.setFillColor(sf::Color::Red);
	s_Main_Title.setCharacterSize(F_SIZE_TITLE);
	// x label
	s_Starttime.setFont(font);
	s_Starttime.setPosition({BORDER_WIDTH, float((g_Config.high - WIN_INDICATOR) + 25)});
	s_Starttime.setFillColor(sf::Color::Red);
	s_Starttime.setCharacterSize(F_SIZE_TIME);
	// right datetime
	s_Datetime.setFont(font);
	s_Datetime.setPosition({float(g_Config.width - RIGHT_WTH), float(g_Config.high - WIN_INDICATOR + 60)});
	s_Datetime.setFillColor(sf::Color::Red);
	s_Datetime.setCharacterSize(F_SIZE_INFO);
	// price
	s_Price.setFont(font);
	s_Price.setPosition({float(g_Config.width - RIGHT_WTH), float(g_Config.high - WIN_INDICATOR + 80)});
	s_Price.setFillColor(sf::Color::Red);
	s_Price.setCharacterSize(F_SIZE_INFO);

	// MACD
	s_Macd1.setFont(font);
	s_Macd1.setPosition({10, 10});
	s_Macd1.setFillColor(sf::Color::Red);
	s_Macd1.setCharacterSize(F_SIZE_TOPIC);
	s_Macd1.setString("MACD: EMA(12)");

	s_Macd2.setFont(font);
	s_Macd2.setPosition({T_MACD_LEFT, 10});
	s_Macd2.setFillColor(sf::Color::Green);
	s_Macd2.setCharacterSize(F_SIZE_TOPIC);
	s_Macd2.setString("EMA(26)");
}

/*
此函数必须放build_lines函数之中，或在或紧跟在后，因为此函数复用了build_lines函数中构造的g_InPrice数据。
目前看此函数的实际价值不大，留待观察。
*/
void talib_RSI(int size, int period)
{
	TA_Integer outRsi_idx;
	TA_Integer outRsi_len;
	g_RSI = 0.0;
	if (size > 30 && period > 3)
	{
		TA_RetCode retCode = TA_RSI(0, size - 1, &g_InPrice[0], period, &outRsi_idx, &outRsi_len, &g_Out1[0]);
		if (retCode != TA_SUCCESS)
		{
			return;
		}
		g_RSI = g_Out1[outRsi_len - 1];
		if (g_RSI >= 80.0 || g_RSI <= 20)
		{
			char s[102];
			snprintf(s, 102, "RSI =  %.2f", g_RSI);
			warning_log("warning", s, 32, 40, 1);
		}
	}
}

/**
 * 绘制价格背景刻度线，时间线和价格线。如果不确定某个位置的情况，可以通过鼠标左键双击来确定（将在后台输出）
 */
void draw_board(sf::RenderWindow &win)
{
	int y2 = g_Config.high - WIN_INDICATOR - 40;
	int y1 = 60;
	sf::Color cc(60, 60, 60);
	double dv = (g_MaxPrice - g_MinPrice) / (y2 - y1);
	int yv = 30;

	for (int y = y2; y > y1; y -= yv)
	{
		sf::Vertex _lines[] = {
			sf::Vertex(sf::Vector2f(BORDER_WIDTH, y), cc),
			sf::Vertex(sf::Vector2f(g_Config.width - BORDER_WIDTH, y), cc)};
		win.draw(_lines, 2, sf::Lines);
		sf::Text sv;
		sv.setFont(g_Font);
		sv.setPosition({float(g_Config.width - 80), float(y - 24)});
		char s[20];
		snprintf(s, 20, "%.2f", g_MinPrice + dv * (y2 - y));
		sv.setString(s);
		sv.setFillColor(sf::Color(180, 180, 180));
		sv.setCharacterSize(F_SIZE_BOARD);
		win.draw(sv);
	}

	// step = g_Config.barsize*6*30 ;

	for (int x = BORDER_WIDTH; x < g_Config.width - RIGHT_WTH - 2 * BORDER_WIDTH; x += g_Config.barsize * 6 * 30)
	{
		sf::Vertex _lines[] = {
			sf::Vertex(sf::Vector2f(x, y2), cc),
			sf::Vertex(sf::Vector2f(x, 80), cc)};
		win.draw(_lines, 2, sf::Lines);

		int idx = (x - BORDER_WIDTH) / g_Config.barsize;
		if (g_Fdata.size() > idx)
		{
			sf::Text sv;
			sv.setFont(g_Font);
			sv.setPosition({float(x), 60});
			string t_s;
			for (auto iter = g_Fdata.begin(); iter != g_Fdata.end(); iter++, idx--)
			{
				if (idx == 0)
				{
					t_s = iter->first;
					break;
				}
			}

			sv.setString(t_s);
			sv.setFillColor(sf::Color(180, 180, 180));
			sv.setCharacterSize(F_SIZE_BOARD);
			win.draw(sv);
		}
	}
}
/**
 * @brief 决策柱状图的绘制
 * 检测10秒内的交易量数据，并用4粒柱方式展现。蓝色为均值，黄色为上上个10秒，绿色为上个10秒，红色表示当前的交易量。
 */
void draw_decision_bar(sf::RenderWindow &win)
{
	int size = g_Volume.size();
	if (size < VOLUME_BAR)
		return;

	if (g_Volume[size - 1] < 0 || g_Volume[size - 2] < 0 || g_Volume[size - 3] < 0)
	{
		if (g_ErrorNo == 0)
			cout << "脏数据，无法正常计算volume , exit" << endl;
		g_ErrorNo = (g_ErrorNo + 1) % (1000);
		return;
	}

	int left = BORDER_WIDTH;
	double x_step = g_Config.barsize;

	auto begin = g_Fdata.begin();
	int vol0 = begin->second.Volume;
	string t_start = begin->first;
	auto end = g_Fdata.end();
	end--;
	int volc = end->second.Volume;
	string t_end = end->first;

	end--;
	int voll = end->second.Volume;
	int tall = tdiff(t_start, t_end);
	if (tall == 0)
		tall = 1;

	// tdiff返回的是秒数，而数据是10个秒的数据，所以乘10
	int vol_avg = (voll - vol0) * 10 / tall;
	if (vol_avg < 0)
		return;

	int avgH = 80;
	int maxH = 360;
	int minH = 50;

	float vol_avg_x = minH + avgH / vol_avg;
	float vol_cur = minH + avgH * g_Volume[size - 1] / vol_avg;
	float vol_last = minH + avgH * g_Volume[size - 2] / vol_avg;
	float vol_last_last = minH + avgH * g_Volume[size - 3] / vol_avg;

	vol_cur = vol_cur > maxH ? maxH + 5 : vol_cur;
	vol_last = vol_last > maxH ? maxH : vol_last;
	vol_last_last = vol_last_last > maxH ? maxH : vol_last_last;

	sf::RectangleShape avbar(sf::Vector2f(16, vol_avg_x));
	avbar.setFillColor(sf::Color::Blue);
	avbar.setPosition(sf::Vector2f(left + x_step * (size + 2) + 30, g_Config.high - WIN_INDICATOR - 100 - vol_avg_x));

	sf::RectangleShape last2bar(sf::Vector2f(16, vol_last_last));
	last2bar.setFillColor(sf::Color::Yellow);
	last2bar.setPosition(sf::Vector2f(left + x_step * (size + 2) + 50, g_Config.high - WIN_INDICATOR - 100 - vol_last_last));

	sf::RectangleShape lastbar(sf::Vector2f(16, vol_last));
	lastbar.setFillColor(sf::Color::Green);
	lastbar.setPosition(sf::Vector2f(left + x_step * (size + 2) + 70, g_Config.high - WIN_INDICATOR - 100 - vol_last));

	sf::RectangleShape curbar(sf::Vector2f(16, vol_cur));
	curbar.setFillColor(sf::Color::Red);
	curbar.setPosition(sf::Vector2f(left + x_step * (size + 2) + 90, g_Config.high - WIN_INDICATOR - 100 - vol_cur));

	win.draw(avbar);
	win.draw(last2bar);
	win.draw(lastbar);
	win.draw(curbar);
}

/**
 * 针对volume lines，表达当前交流量的大小关系。在整个窗口的最低部。超过1/3高度的，按量画红点标注。
 */
void indicator_volume(sf::RenderWindow &win)
{
	int size = g_Volume.size();
	if (size == 0)
		return;

	int sum = accumulate(g_Volume.begin(), g_Volume.end(), 0);
	float avg = sum / (size * g_Config.volscale);
	for (int i = 0; i < size; i++)
	{
		float vol = g_Volume[i] / g_Config.volscale;
		if (vol < 2 * avg) // 太小了就不画了，没意思
			continue;

		sf::Color color = sf::Color(255, 255, 0, 200);
		if (vol > (g_Config.high - 22 - WIN_INDICATOR))
		{
			color = sf::Color(250, 0, 0, 200);
			vol = g_Config.high - 22 - WIN_INDICATOR;
		}

		float x = 10 + i * g_Config.barsize;
		float y = g_Config.high - 20;

		sf::Vertex _lines[] = {
			sf::Vertex(sf::Vector2f(x, y), color),
			sf::Vertex(sf::Vector2f(x, y - vol), color)};
		win.draw(_lines, 2, sf::Lines);

		if (vol > (WIN_INDICATOR - 40) / 3)
		{
			float r = vol * 8 / (WIN_INDICATOR - 40);
			sf::CircleShape cc(r);
			cc.setFillColor(sf::Color::Red);
			cc.setPosition({float(x) - r, float(y - (WIN_INDICATOR - 40) / 2)});
			win.draw(cc);
		}
	}
}

/**
 * 计算Volume值，并存储到 g_Volume 中。
 *
 */
void build_volume_vector(bool only_last)
{
	int size = g_Fdata.size();
	if (size == 0)
		return;
	else if (size == 1)
	{
		g_Volume.clear();
		g_Volume.push_back(1); // 不确定数据是否从0开始切入，为了防止中途切入的数据，故第一个初始值为1。
	}
	else
	{ // 此时，数据至少 1 条。
		// cout << "size: " << size << "; volume size = "<< g_Volume.size() << endl;
		if (only_last)
		{ // 逐条增加的正常运行时逻辑
			auto it = g_Fdata.end();
			it--; // 当前数据，
			int v0 = it->second.Volume;
			SimpleMd sm = it->second;
			string s_time = it->first;
			//			float p = it->second.LastPrice;
			it--;
			int v1 = it->second.Volume;
			int vv = v0 - v1;

			if (size == g_Volume.size() + 1)
			{
				g_Volume.push_back(vv);
			}
			else if (size == g_Volume.size())
			{
				g_Volume[size - 1] = vv;
			}
			if (g_Volume.size() < 3)
				return;
			//  int last_v = g_Volume[g_Volume.size() - 2];
			g_TopV.insert(vv, s_time);

			if ((g_TopV.is_top(vv) || vv > g_Config.volwarning) && g_Config.strategy == StrategyType::VOLUME)
			{
				char s[100];
				snprintf(s, 100, "飚车开始，做好准备: %s : Price = %.2f， Volume = %d", s_time.c_str(), sm.LastPrice, vv);
				mylog("TOPV", s, 31, 42, 0);
				run_volume_strategy();
			}
		}
		else if (g_Volume.size() == 0)
		{ // 第一次文件导入数据的逻辑
			int i = 0;
			int last = 0;

			for (map<string, SimpleMd>::iterator it = g_Fdata.begin(); it != g_Fdata.end(); ++it, i++)
			{
				if (it == g_Fdata.begin())
				{
					g_Volume.push_back(1);
					last = it->second.Volume;
				}
				else
				{
					float vv = it->second.Volume - last;
					g_Volume.push_back(vv);
					last = it->second.Volume;

					string s_time = it->first;
					g_TopV.insert(vv, s_time);
				}
			}
		}
	}
}

/**
 * 获取文件名（根据时间和商品编号）
 */
string get_file_name(const char *futuresId, string year_of_day = "")
{
	if (year_of_day.length() == 0)
		year_of_day = get_year_of_day();
	string fname = "./data/" + string(futuresId) + "-" + year_of_day + ".txt";
	return fname;
}

/**
 * 获取此刻年月日（%Y%m%d）格式的字符串
 */
string get_year_of_day()
{
	auto t2 = system_clock::now();
	auto t_now = system_clock::to_time_t(t2);
	std::tm tm2 = {0};
	localtime_r(&t_now, &tm2); // linux线程安全, windows is localtime_t()
	char now_str[32];
	strftime(now_str, 32, "%Y%m%d", &tm2);
	return now_str;
}

void save_data(string year_of_day)
{
	if (!file_exist("./data"))
	{
		mkdir("./data", 0777);
	}
	string fname = get_file_name(g_Config.fName.c_str(), year_of_day);
	if (file_exist(fname.c_str()))
	{
		remove(fname.c_str());
	}
	ofstream ofs;
	ofs.open(fname, ios::out);
	for (auto it = g_Fdata.begin(); it != g_Fdata.end(); ++it)
	{
		ofs << it->first << "," << it->second.LastPrice << "," << it->second.AvgPrice << "," << it->second.Volume << "," << it->second.Radius << "\n";
	}
	ofs.close();
}

bool load_data_from_bunny(string year_of_day)
{

	string datas = read_ticks_from_server(g_Config.bunny, g_Config.fName, year_of_day);
	if (datas.length() == 0)
		cout << "read from bunny, but none" << endl;

	vector<string> lines = stringSplit(datas, '\n');
	for (auto line : lines)
	{
		vector<string> cols = stringSplit(line, ',');
		if (cols.size() == 4)
		{
			SimpleMd md;
			md.LastPrice = atof(cols[1].c_str());
			md.AvgPrice = atof(cols[2].c_str());
			md.Volume = atoi(cols[3].c_str());
			md.Radius = 0;
			g_Fdata.insert(pair<string, SimpleMd>(cols[0], md));

			if (g_OpenPrice == 0)
			{
				g_OpenPrice = md.LastPrice;
			}
			if (g_Datetime.size() == 0)
			{
				g_Datetime = year_of_day;
			}

			double minp = min(md.AvgPrice, md.LastPrice);
			if (minp == 0)
			{
				cout << "average = 0, error" << endl;
				minp = (int(md.LastPrice) / 10) * 10;
				md.AvgPrice = minp;
			}
			if (g_MinPrice == 0.0)
			{
				g_MinPrice = minp - 2;
			}
			else if (g_MinPrice > minp)
				g_MinPrice = minp - 2;

			double maxp = max(md.AvgPrice, md.LastPrice);
			if (g_MaxPrice < maxp)
				g_MaxPrice = maxp + 2;
		}
	}
	return g_Fdata.size() > 0;
}

void load_data(string year_of_day)
{
	bool remote = false;
	if (g_Config.bunny.length() > 0)
		remote = load_data_from_bunny(year_of_day);

	if (remote == false)
	{

		fstream ifs;
		string fname = get_file_name(g_Config.fName.c_str(), year_of_day);
		cout << "read from local file " << fname << endl;
		ifs.open(fname, ios::in);
		if (!ifs.is_open())
		{
			cout << "文件不存在，或打开失败:" << fname << endl;
			return; // 失败结束
		}
		// 读数据
		// char buf[1024] = {0};
		string str;
		while (getline(ifs, str)) //          (ifs >> buf) // 如果文件结束，会返回结束标志，退出while
		{
			if (str.length() < 10)
				continue;
			vector<string> v = stringSplit(str.c_str(), ',');

			if (v.size() == 5)
			{
				SimpleMd md;
				md.LastPrice = atof(v[1].c_str());
				md.AvgPrice = atoi(v[2].c_str());
				md.Volume = atoi(v[3].c_str());
				md.Radius = atoi(v[4].c_str());
				g_Fdata.insert(pair<string, SimpleMd>(v[0], md));

				double minp = min(md.AvgPrice, md.LastPrice);
				if (minp == 0)
				{
					cout << "average = 0" << endl;
					minp = (int(md.LastPrice) / 10) * 10;
					md.AvgPrice = minp;
				}
				if (g_MinPrice == 0.0)
				{
					g_MinPrice = minp - 2;
				}
				else if (g_MinPrice > minp)
					g_MinPrice = minp - 2;

				double maxp = max(md.AvgPrice, md.LastPrice);
				if (g_MaxPrice < maxp)
					g_MaxPrice = maxp + 2;

				if (g_OpenPrice == 0)
				{
					g_OpenPrice = md.LastPrice;
				}
				if (g_Datetime.size() == 0)
				{
					g_Datetime = "unkown";
				}
			}
		}

		ifs.close();
	}
}

void run_analyse()
{

	if (!g_Volume.empty())
	{
		g_TopV.print();
		cout << "\n当前流量：" << g_Volume[g_Volume.size() - 1] << "，当前价格：" << g_InPrice[g_Volume.size() - 1] << "\n";
	}
	cout << "\n- 1. 第一单叫机会，第二单叫改命，第三单，叫做救命！\n";
	cout << "- 2. 小探如钓鱼，连续引诱必有诈；大单如馅饼，可遇不可求！\n";
	cout << "- 3. 纪律，点位特征？不要为眼前的小利而冲动！" << endl;
}
void print_volume()
{
	if (g_Fdata.empty())
	{
		cout << "volume is empty" << endl;
		return;
	}
	cout << "volume is:\n";
	int last = g_Fdata.begin()->second.Volume - 1;
	for (auto it = g_Fdata.begin(); it != g_Fdata.end(); it++)
	{
		std::cout << it->first << ":	" << it->second.Volume - last << ",	" << it->second.LastPrice << "\n";
		last = it->second.Volume;
	}
	cout << endl;
}

/*
if Volume is FiveMinuteTOP and Volume > g_Config.volwarning, then send signal to buy;

*/
bool run_volume_strategy()
{
	bool ret = false;
	int size = g_Volume.size();
	if (size > MACD_EMA_BIG)
	{
		cout << "------   run volume strategy   ------\n";
		// 26: 14: 0
		int f_idx = size - MACD_EMA_SMALL;
		int s_idx = size - MACD_EMA_BIG;
		auto it = g_Fdata.end();
		it--;
		string ts = it->first;
		stringstream ss;
		double diff_v = 0.0;
		if (g_DiffPrice < 0.0)
		{
			ss << "wish to DOWN! ";
			diff_v = (-1 * g_DiffPrice) - it->second.LastPrice;
			if (diff_v < g_Config.dvalue)
			{
				ss << to_string(it->second.LastPrice) << " is still higher";
				if (g_Out1[f_idx] >= g_Out2[s_idx])
					ss << "; Pay attention, Now is Uping!";
				cout << ss.str() << endl;
				return ret;
			}
		}
		else if (g_DiffPrice > 0.0)
		{
			ss << "wish to UP! ";
			diff_v = it->second.LastPrice - g_DiffPrice;
			if (diff_v < g_Config.dvalue)
			{
				ss << to_string(it->second.LastPrice) << " is still lower";
				if (g_Out1[f_idx] >= g_Out2[s_idx])
					ss << "; Pay attention, Now is Downing!";
				cout << ss.str() << endl;
				return ret;
			}
		}
		else
		{
			ss << "NO ding data! ";
			if (it->second.LastPrice > it->second.AvgPrice)
				diff_v = it->second.LastPrice - it->second.AvgPrice;
			else if (it->second.LastPrice < it->second.AvgPrice)
				diff_v = it->second.AvgPrice - it->second.LastPrice;

			if (diff_v < g_Config.dvalue)
			{
				ss << to_string(it->second.LastPrice) << " is too center-small";
				cout << ss.str() << endl;
				return ret;
			}
		}

		if (g_Out1[f_idx] >= g_Out2[s_idx] && it->second.LastPrice > it->second.AvgPrice) // 上涨姿态
		{
			ss << ts << ": 开空/平多/反手：" << to_string(it->second.LastPrice) << ", DiffV = " << to_string(diff_v);
			printf("%c[%d;%d;%dm[%s]%c[0m %c[%d;%d;%dm%s%c[0m\n", 0x1B, 0, 40, 31, "TOPV", 0x1B, 0x1B, 0, 47, 31, ss.str().c_str(), 0x1B);
			ret = true;
			if (g_Config.strategy == StrategyType::VOLUME)
				if (g_Strategy.Sell(it->second.LastPrice, it->first))
				{

					it->second.Radius = -10.0;
					g_DiffPrice = 0.0; //-1*it->second.LastPrice;
				}
		}
		else if (g_Out1[f_idx] < g_Out2[s_idx] && it->second.LastPrice < it->second.AvgPrice) // 下跌姿态
		{
			stringstream ss;
			ss << ts << ": 开多/平空/反手：" << to_string(it->second.LastPrice) << ", DiffV = " << to_string(diff_v);
			printf("%c[%d;%d;%dm[%s]%c[0m %c[%d;%d;%dm%s%c[0m\n", 0x1B, 0, 40, 31, "TOPV", 0x1B, 0x1B, 0, 47, 32, ss.str().c_str(), 0x1B);
			ret = true;
			if (g_Config.strategy == StrategyType::VOLUME)
				if (g_Strategy.Buy(it->second.LastPrice, it->first))
				{

					it->second.Radius = -10.0;
					g_DiffPrice = 0.0; // it->second.LastPrice;
				}
		}

		cout << ss.str() << endl;
	}
	return ret;
}

/*
MACD signal strategy
*/
bool run_macd_strategy() // double macd1, double macd2)
{
	int size = g_Fdata.size();
	bool ret = false;
	if (size > MACD_EMA_BIG)
	{
		cout << "------   run macd strategy   ------\n";
		// 26: 14: 0
		int f_idx = size - MACD_EMA_SMALL;
		int s_idx = size - MACD_EMA_BIG;
		auto it = g_Fdata.end();
		it--;
		string ts = it->first;
		int vsize = g_Volume.size();

		// 过去的x个bar内是否有高流量发生, 按秒计则取10, 按10秒计则取 3
		int last = g_Config.seconds ? 10 : 3;
		int vol = *max_element(&g_Volume[vsize - last], &g_Volume[size]);
		// 获取最大volume报警值。
		int bvol = g_Config.seconds ? g_Config.volwarning : g_Config.volwarning * 10;

		// 红在绿上， 交易量超过阈值
		if (g_Out1[f_idx] > g_Out2[s_idx] && vol >= bvol) // 上涨姿态
		{
			double diffv = g_DiffPrice > g_MinPrice ? it->second.LastPrice - g_DiffPrice : it->second.LastPrice - it->second.AvgPrice;
			stringstream ss;
			ss << ts << ": 开空/平多/反手：" << to_string(it->second.LastPrice) << ", DiffV = " << to_string(diffv);
			printf("%c[%d;%d;%dm[%s]%c[0m %c[%d;%d;%dm%s%c[0m\n", 0x1B, 0, 40, 31, "MACD", 0x1B, 0x1B, 4, 47, 31, ss.str().c_str(), 0x1B);
			ret = true;
			if (g_Config.strategy == StrategyType::MACD)
			{
				if (g_Strategy.Sell(it->second.LastPrice, it->first))
				{

					it->second.Radius = -10.0;
					g_DiffPrice = 0.0; // -1* it->second.LastPrice;
				}
				else
				{
					cout << "strategy failed!" << endl;
				}
			}
		}													   //  红在绿下， 交易量超过阈值，在均价之下
		else if (g_Out1[f_idx] < g_Out2[s_idx] && vol >= bvol) // 下跌姿态
		{
			double diffv = g_DiffPrice < -1 * g_MinPrice ? it->second.LastPrice + g_DiffPrice : it->second.AvgPrice - it->second.LastPrice;
			stringstream ss;
			ss << ts << ": 开多/平空/反手：" << to_string(it->second.LastPrice) << ", DiffV = " << to_string(diffv);
			printf("%c[%d;%d;%dm[%s]%c[0m %c[%d;%d;%dm%s%c[0m\n", 0x1B, 0, 40, 31, "MACD", 0x1B, 0x1B, 4, 47, 31, ss.str().c_str(), 0x1B);
			ret = true;
			if (g_Config.strategy == StrategyType::MACD)
			{
				if (g_Strategy.Buy(it->second.LastPrice, it->first))
				{
					it->second.Radius = -10.0;
					g_DiffPrice = 0.0; // it->second.LastPrice;
				}
				else
				{
					cout << "strategy failed!" << endl;
				}
			}
		}
		else
		{
			stringstream ss;
			ss << ts << ": 流量太小, 建议放过 ～ ， *max_element(&vol[-6], vol[-1]) = " << vol << ", but config.volwarning = " << bvol;
			printf("%c[%d;%d;%dm[%s]%c[0m %c[%d;%d;%dm%s%c[0m\n", 0x1B, 0, 40, 31, "MACD", 0x1B, 0x1B, 4, 47, 34, ss.str().c_str(), 0x1B);
		}
		cout << endl;
	}
	return ret;
}

void print_help()
{
	stringstream ss;
	ss << "\n-----  This is futures monitor by @lengss   -----\n\n";
	ss << " 1. 鼠标左键双击，打印当前鼠标位置对应数据信息；\n";
	ss << " 2. 鼠标先左后右键点击，在当前位置增加小红旗；\n";
	ss << " 3. 鼠标右键双击；删除所在位置小红旗；\n";
	ss << " 4. F1: 打印帮助信息;\n";
	ss << " 5. F2: 打印Strategy数据;\n";
	ss << " 6. F3: 打印所有Volume数据;\n";
	ss << " 7. Delete: 删除所有数据；\n";
	ss << " 8. Space: 唤起分析程序, 打印当前TOP情况;\n";
	ss << " 9. q: q键退出程序;\n\n";

	double cur = 0.0;
	if (!g_Fdata.empty())
	{
		auto it = g_Fdata.end();
		it--;
		cur = it->second.LastPrice;
	}
	if (g_DiffPrice >= g_MinPrice)
	{
		ss << "当前盯涨势：" << to_string(g_DiffPrice) << ", 最新价位：" << to_string(cur) << ", 差价：" << to_string(cur - g_DiffPrice) << "\n\n";
	}
	else if (g_DiffPrice < -1 * g_MinPrice)
	{
		ss << "当前盯跌势：" << to_string(-1 * g_DiffPrice) << ", 最新价位：" << to_string(cur) << ", 差价：" << to_string(-1 * (g_DiffPrice + cur)) << "\n\n";
	}
	else
	{
		ss << "当前无盯价操作！系统以均价为标准。也可以鼠标左键双击,然后按'+'盯涨，按'-'盯跌！\n\n";
	}
	cout << ss.str();
}

void print_strategy()
{
	g_Strategy.Print();
}
