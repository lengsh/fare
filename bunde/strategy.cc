#include "strategy.h"

using namespace std;


string trade_string(const TradeType &obj)
{
	stringstream os;
	switch (obj)
	{
	case KDUO:
		os << "开多";
		break;
	case KKONG:
		os << "开空";
		break;
	case PDUO:
		os << "平多";
		break;
	case PKONG:
		os << "平空";
		break;
	case FAN:
		os << "反手";
		break;
	default:
		break;
	}
	return os.str();
}

/**
 * Buy: KDUO, PKONG
 */
bool Strategy::Buy(double price, string stime)
{
	bool ret = false;
	if (m_records.empty())
	{
		Trade t = Trade{m_num, price, stime, TradeType::KDUO};
		m_records.push_back(t);
		ret = true;
	}
	else
	{
		switch (m_records[m_records.size() - 1].ttype)
		{
		case KDUO:
			break;
		case KKONG:
			m_records.push_back(Trade{m_num, price, stime, TradeType::PKONG});
			ret = true;
			break;
		case PDUO:
		case PKONG:
			m_records.push_back(Trade{m_num, price, stime, TradeType::KDUO});
			ret = true;
			break;
		default:
			cerr << "not support" << endl;
			break;
		}
	}
	return ret;
}

/**
 * Sell: KKONG, PDUO
 */
bool Strategy::Sell(double p, string st)
{
	bool ret = false;
	if (m_records.empty())
	{
		m_records.push_back(Trade{m_num, p, st, TradeType::KKONG});
		ret = true;
	}
	else
	{
		switch (m_records[m_records.size() - 1].ttype)
		{
		case KDUO:
			m_records.push_back(Trade{m_num, p, st, TradeType::PDUO});
			ret = true;
			break;
		case KKONG:
			break;
		case PDUO:
		case PKONG:
			m_records.push_back(Trade{m_num, p, st, TradeType::KKONG});
			ret = true;
			break;
		default:
			cerr << "not support" << endl;
			break;
		}
	}
	return ret;
}

/**
 * Fan shou
 */
bool Strategy::Fan(double p, string st)
{
	bool ret = false;
	if (!m_records.empty())
	{
		switch (m_records[m_records.size() - 1].ttype)
		{
		case KDUO:
			m_records.push_back(Trade{m_num, p, st, TradeType::PDUO});
			m_records.push_back(Trade{m_num, p, st, TradeType::KKONG});
			ret = true;
			break;
		case KKONG:
			m_records.push_back(Trade{m_num, p, st, TradeType::PKONG});
			m_records.push_back(Trade{m_num, p, st, TradeType::KDUO});
			ret = true;
			break;
		case PDUO:
		case PKONG:
			break;
		default:
			cerr << "not support" << endl;
			break;
		}
	}
	return ret;
}
/**
 * KKONG, PDUO, FAN
*/
bool Strategy::HighAction(double p, string st)
{
	bool ret = false;
	if (m_records.empty()){
			m_records.push_back(Trade{m_num, p, st, TradeType::KKONG});			
			ret = true;
	}
	else
	{
		switch (m_records[m_records.size() - 1].ttype)
		{
		case KDUO:
			m_records.push_back(Trade{m_num, p, st, TradeType::PDUO});
			m_records.push_back(Trade{m_num, p, st, TradeType::KKONG});
			ret = true;
			break;
		case KKONG:
			break;
		case PDUO:
		case PKONG:
			m_records.push_back(Trade{m_num, p, st, TradeType::KKONG});
			ret = true;
			break;
		default:
			cerr << "not support" << endl;
			break;
		}
	}
	return ret;
}

/**
 * KDUO, PKONG, FAN
*/
bool Strategy::LowAction(double p, string st)
{
	bool ret = false;

	if (m_records.empty()){
			m_records.push_back(Trade{m_num, p, st, TradeType::KDUO});			
			ret = true;
	}
	else
	{
		switch (m_records[m_records.size() - 1].ttype)
		{
		case KDUO:		
			ret = false;
			break;
		case KKONG:
			m_records.push_back(Trade{m_num, p, st, TradeType::PKONG});
			m_records.push_back(Trade{m_num, p, st, TradeType::KDUO});
			ret = true;
			break;
		case PDUO:
		case PKONG:			
			m_records.push_back(Trade{m_num, p, st, TradeType::KDUO});
			ret = true;
			break;		
		default:
			cerr << "not support" << endl;
			break;
		}
	}
	return ret;
}


void Strategy::Clear()
{
	m_records.clear();
}

double Strategy::Calculate()
{	
	double t_f = 30.0;
	double t_sum = 0.0;
	double t_sub = 0.0;

	int i = 0; 
	int j = m_records.size();
	for (i=0; i< j; i += 2){
		double a = m_records[i].Price;
		double b = m_records[i+1].Price;
		if (m_records[i+1].ttype == TradeType::PDUO ){
			t_sum += ((b-a)*10*m_num);
		}
		if (m_records[i+1].ttype == TradeType::PKONG ){
			t_sum += ((a-b)*10*m_num);
		}
		t_sub += t_f*m_num;
	}
	cout <<"收益: "<< (t_sum - t_sub) <<"; 佣金+手续费: "<< t_sub << endl;
	return t_sum;
}
void Strategy::Print()
{
	if (this->m_records.empty())
		cout << "Trade records is empty!" << endl;
	else
	{
		cout << "Trade records as :\n";
		for (auto x : m_records)
		{
			cout << x.Time <<": "<< trade_string(x.ttype) <<", " << x.Price << " * "<< x.Volume << "\n";
		}
		cout << endl;
	}
	Calculate();
}
