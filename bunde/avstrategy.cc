#include "strategy.h"

using namespace std;

void AVStrategy::Clear()
{
	m_Avs.clear();
}

bool AVStrategy::Next(double now, double av, string st)
{
	bool ret = false;
	int signal = (int)(now - av) / m_Value;
	if (m_Avs.empty())
	{
		m_Avs.push_back(AVsignal{signal, now, st});
		ret = true;
	}
	else
	{ //  只有新增变化加大或者变成反方向，才加一个策略
		if (((signal > 0 && signal > m_Avs[m_Avs.size() - 1].signal) ||
			 (signal < 0 && signal < m_Avs[m_Avs.size() - 1].signal) ||
			 (signal > 0 && m_Avs[m_Avs.size() - 1].signal < 0) ||
			 (signal < 0 && m_Avs[m_Avs.size() - 1].signal > 0)) &&
			st.compare(m_Avs[m_Avs.size() - 1].Time) > 0)
		{
			m_Avs.push_back(AVsignal{signal, now, st});
			ret = true;
		}
	}

	if (ret)
	{
		stringstream ss;
		if (signal > 10 || signal < -10)
			ss << "崩盘了，止损跑路吧！";

		switch (signal)
		{
		case 1:
		case 2:
			ss << "第一张牌: 开空/平多";
			break;

		case 3:
		case 4:
			ss << "第二张牌(慎重): 追加 开空";
			break;

		case 5:
		case 6:
		case 7:
		case 8:
		case 9:
		case 10:
			ss << "第三张牌(绝望慎重): 拼死 开空";
			break;
		case -1:
		case -2:
			ss << "第一张牌: 开多/平空";
			break;

		case -3:
		case -4:
			ss << "第二张牌: 追加 开多";
			break;
		case -5:
		case -6:
		case -7:
		case -8:
		case -9:
		case -10:
			ss << "第三张牌: 拼死开多";
			break;
		default:
			break;
		}
		if (ss.str().length() > 0)
		{
			ss << "; 价位: " << to_string(now) << "; 均值差: " << to_string(abs(now - av)) << ", 时间: " << st;
			printf("%c[%d;%d;%dm[%s]%c[0m %c[%d;%d;%dm%s%c[0m\n", 0x1B, 0, 40, 31, "AVStrategy", 0x1B, 0x1B, 0, 40, 31, ss.str().c_str(), 0x1B);
		}
	}

	return ret;
}
int AVStrategy::GetSignal(int idx)
{
	int size = m_Avs.size();
	if (idx < 0 && size >= -1 * idx)
		return m_Avs[size + idx].signal;
	else
		return 0;
	;
}