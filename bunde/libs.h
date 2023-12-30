#include <iostream>
#include <vector>
#include <chrono>
#include <map>
#include <sstream>
#include <ta-lib/ta_libc.h>
#include <ta-lib/ta_func.h>
using namespace std;

#ifndef _LIB_H
#define _LIB_H
#endif

int hello_talib();
bool file_exist(const char *filename);
// std::chrono::system_clock::time_point  warning_last = system_clock::now();
/**
 *
const (
	//  前景字符颜色
	F_RED    F_Color = 31
	F_BLUE   F_Color = 34
	F_GREEN  F_Color = 32
	F_YELLOW F_Color = 33
	F_BLACK  F_Color = 30
	F_WHITE  F_Color = 37
	F_CYAN   F_Color = 36
	F_PURPLE F_Color = 35

	// 后景背景颜色
	B_RED    B_Color = 41
	B_BLUE   B_Color = 44
	B_GREEN  B_Color = 42
	B_YELLOW B_Color = 43
	B_BLACK  B_Color = 40
	B_WHITE  B_Color = 47
	B_CYAN   B_Color = 46
	B_PURPLE B_Color = 45
	// 0 终端默认设置 // 1 高亮显示 // 4 使用下划线 // 5 闪烁 // 7 反白显示 // 8 不可见
	S_TERM S_Color = 0
	S_HIGH S_Color = 1
	S_LINE S_Color = 4
	S_TWIN S_Color = 5
	S_UWHI S_Color = 7
	//	S_NOSH    S_Color = 8
)
*/
void mylog(const char *header, const char *log, int color, int bcolor, int style);
void warning_log(const char *header, const char *log, int color = 31, int bcolor = 47, int style = 0);
int tdiff(string s, string b);
bool norepeat();

/*
 * {
printf("%s: %c[%d;%d;%dm%s%c[0m\n",header, 0x1B, 5, 46, 31, log, 0x1B);
}
*/
/*
typedef char TThostFtdcDateType[9];
typedef char TThostFtdcOldInstrumentIDType[31];
typedef char TThostFtdcExchangeIDType[9];
typedef char TThostFtdcOldExchangeInstIDType[31];
typedef double TThostFtdcPriceType;
typedef double TThostFtdcLargeVolumeType;
typedef int TThostFtdcVolumeType;
typedef char TThostFtdcInstrumentIDType[81];
typedef double TThostFtdcRatioType;
typedef char TThostFtdcTimeType[9];
typedef char TThostFtdcExchangeInstIDType[81];
typedef double TThostFtdcMoneyType;
typedef int TThostFtdcMillisecType;
///�������
struct DepthMarketData
{
	///������
	TThostFtdcDateType	TradingDay;
	///��������Ч�ֶ�
	TThostFtdcOldInstrumentIDType	reserve1;
	///����������
	TThostFtdcExchangeIDType	ExchangeID;
	///��������Ч�ֶ�
	TThostFtdcOldExchangeInstIDType	reserve2;
	///���¼�
	TThostFtdcPriceType	LastPrice;
	///�ϴν����
	TThostFtdcPriceType	PreSettlementPrice;
	///������
	TThostFtdcPriceType	PreClosePrice;
	///��ֲ���
	TThostFtdcLargeVolumeType	PreOpenInterest;
	///����
	TThostFtdcPriceType	OpenPrice;
	///��߼�
	TThostFtdcPriceType	HighestPrice;
	///��ͼ�
	TThostFtdcPriceType	LowestPrice;
	///����
	TThostFtdcVolumeType	Volume;
	///�ɽ����
	TThostFtdcMoneyType	Turnover;
	///�ֲ���
	TThostFtdcLargeVolumeType	OpenInterest;
	///������
	TThostFtdcPriceType	ClosePrice;
	///���ν����
	TThostFtdcPriceType	SettlementPrice;
	///��ͣ���
	TThostFtdcPriceType	UpperLimitPrice;
	///��ͣ���
	TThostFtdcPriceType	LowerLimitPrice;
	///����ʵ��
	TThostFtdcRatioType	PreDelta;
	///����ʵ��
	TThostFtdcRatioType	CurrDelta;
	///����޸�ʱ��
	TThostFtdcTimeType	UpdateTime;
	///����޸ĺ���
	TThostFtdcMillisecType	UpdateMillisec;
	///�����һ
	TThostFtdcPriceType	BidPrice1;
	///������һ
	TThostFtdcVolumeType	BidVolume1;
	///������һ
	TThostFtdcPriceType	AskPrice1;
	///������һ
	TThostFtdcVolumeType	AskVolume1;
	///����۶�
	TThostFtdcPriceType	BidPrice2;
	///��������
	TThostFtdcVolumeType	BidVolume2;
	///�����۶�
	TThostFtdcPriceType	AskPrice2;
	///��������
	TThostFtdcVolumeType	AskVolume2;
	///�������
	TThostFtdcPriceType	BidPrice3;
	///��������
	TThostFtdcVolumeType	BidVolume3;
	///��������
	TThostFtdcPriceType	AskPrice3;
	///��������
	TThostFtdcVolumeType	AskVolume3;
	///�������
	TThostFtdcPriceType	BidPrice4;
	///��������
	TThostFtdcVolumeType	BidVolume4;
	///��������
	TThostFtdcPriceType	AskPrice4;
	///��������
	TThostFtdcVolumeType	AskVolume4;
	///�������
	TThostFtdcPriceType	BidPrice5;
	///��������
	TThostFtdcVolumeType	BidVolume5;
	///��������
	TThostFtdcPriceType	AskPrice5;
	///��������
	TThostFtdcVolumeType	AskVolume5;
	///���վ���
	TThostFtdcPriceType	AveragePrice;
	///ҵ������
	TThostFtdcDateType	ActionDay;
	///��Լ����
	TThostFtdcInstrumentIDType	InstrumentID;
	///��Լ�ڽ������Ĵ���
	TThostFtdcExchangeInstIDType	ExchangeInstID;
	///�ϴ���
	TThostFtdcPriceType	BandingUpperPrice;
	///�´���
	TThostFtdcPriceType	BandingLowerPrice;
};


*/
