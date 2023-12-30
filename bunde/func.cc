#include <chrono>
#include <ctime>
#include <time.h>
#include <sys/stat.h>
#include <sstream>
#include <iomanip>
#include "libs.h"

using namespace std;

std::chrono::system_clock::time_point warning_last = std::chrono::system_clock::now();
std::chrono::system_clock::time_point norepeat_last = std::chrono::system_clock::now();
int hello_talib()
{
	// Technical analysis part of the code
	TA_Real closePrice[40];
	TA_Real out[40];
	TA_Integer outBeg;
	TA_Integer outNbElement;
	/* ... initializeyour closing price here... */
	for (int i = 0; i < 40; i++)
	{
		closePrice[i] = 1.0 * (i % 8);
	}

	printf("TA-Lib correctly initialized.\n");
	TA_RetCode retCode = TA_MA(0, 39, &closePrice[0], 30, TA_MAType_SMA, &outBeg, &outNbElement, &out[0]);
	for (int i = 0; i < outNbElement; i++)
		printf("Day %d = %f\n", outBeg + i, out[i]);
	return 0;
}

void mylog(const char *header, const char *log, int color, int bcolor, int style)
{
	auto t_now = std::chrono::system_clock::to_time_t(warning_last);
	std::tm tm = {0};
	localtime_r(&t_now, &tm); // linux线程安全, windows is localtime_t()
	char now_str[32];
	strftime(now_str, 32, "%Y%m%d %H:%M:%S", &tm);
	printf("%c[%d;%d;%dm[%s]%c[0m:[%s] %c[%d;%d;%dm%s%c[0m\n", 0x1B, 0, 40, 31, header, 0x1B, now_str, 0x1B, style, bcolor, color, log, 0x1B);
}

void warning_log(const char *header, const char *log, int color, int bcolor, int style)
{
	auto rsptime = std::chrono::system_clock::now();
	chrono::seconds sec = chrono::duration_cast<chrono::seconds>(rsptime - warning_last);
	if (sec.count() > 30)
	{
		mylog(header, log, color, bcolor, style);
		warning_last = rsptime; //  std::chrono::system_clock::now();
	}
}

bool norepeat()
{
	auto rsptime = std::chrono::system_clock::now();
	chrono::seconds sec = chrono::duration_cast<chrono::seconds>(rsptime - norepeat_last);
	if (sec.count() > 30)
	{
		norepeat_last = rsptime;
		return true;
	}
	return false;
}

bool file_exist(const char *filename)
{
	struct stat buffer;
	return stat(filename, &buffer) == 0;
}

int tdiff(string t1, string t2)
{
	auto t = std::chrono::system_clock::now();
	auto t_now = std::chrono::system_clock::to_time_t(t);
	std::tm *tm = localtime(&t_now); // 使用本地时区

	std::stringstream cur;				  // 创建stringstream对象 ss,需要包含<sstream>头文件
	cur << std::put_time(tm, "%Y-%m-%d"); // << " " << t1;
	string a = cur.str() + " " + t1;
	string b = cur.str() + " " + t2;

	std::tm tm_a;
	time_t tt_a;
	strptime(a.c_str(), "%Y-%m-%d %H:%M:%S", &tm_a); // 将字符串转换为tm时间
	tm_a.tm_isdst = -1;
	tt_a = mktime(&tm_a); // 将tm时间转换为秒时间  

	std::tm tm_b;
	time_t tt_b;
	strptime(b.c_str(), "%Y-%m-%d %H:%M:%S", &tm_b); // 将字符串转换为tm时间
	tm_b.tm_isdst = -1;
	tt_b = mktime(&tm_b); // 将tm时间转换为秒时间  

	std::tm tm_s1;
	time_t tt_s1;
	string ss = cur.str() + " 10:15:00";
	strptime(ss.c_str(), "%Y-%m-%d %H:%M:%S", &tm_s1); // 将字符串转换为tm时间
	tm_s1.tm_isdst = -1;
	tt_s1 = mktime(&tm_s1); // 将tm时间转换为秒时间  

	std::tm tm_s2;
	time_t tt_s2;
	ss = cur.str() + " 10:30:00";
	strptime(ss.c_str(), "%Y-%m-%d %H:%M:%S", &tm_s2); // 将字符串转换为tm时间
	tm_s2.tm_isdst = -1;
	tt_s2 = mktime(&tm_s2); // 将tm时间转换为秒时间  

	std::tm tm_z1;
	time_t tt_z1;
	ss = cur.str() + " 11:30:00";
	strptime(ss.c_str(), "%Y-%m-%d %H:%M:%S", &tm_z1); // 将字符串转换为tm时间
	tm_z1.tm_isdst = -1;
	tt_z1 = mktime(&tm_z1); // 将tm时间转换为秒时间  

	std::tm tm_z2;
	time_t tt_z2;
	ss = cur.str() + " 13:30:00";
	strptime(ss.c_str(), "%Y-%m-%d %H:%M:%S", &tm_z2); // 将字符串转换为tm时间
	tm_z2.tm_isdst = -1;
	tt_z2 = mktime(&tm_z2); // 将tm时间转换为秒时间  

	int len = (int)(tt_b - tt_a);
	if (tt_a <= tt_s1 && tt_b >= tt_s2)
	{
		len = len - (int)(15 * 60);
	}
	else
	{
		// cout<< tt_a <<";"<<tt_s1<<";"<<tt_s2<<";"<<tt_b<<endl;
	}
	if (tt_a <= tt_z1 && tt_b >= tt_z2)
		len = len - (int)(2 * 60 * 60);
	return len;
}
