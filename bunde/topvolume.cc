#include "strategy.h"

using namespace std;

bool TopVolume::insert(int vol, string dt)
{
	if (vol <= m_min || dt.length() < 5)
		return false;
	string key = dt.substr(0, 4);
	// is exist
	if (m_topT.find(key) != m_topT.end())
	{
		int v = m_topT[key];
		if (vol > v)
		{
			m_topT[key] = vol;
			auto it = m_topV.find(v);
			if (it != m_topV.end())
				m_topV.erase(it);
			m_topV.insert(make_pair(vol, key));
			return true;
		}
		return false;
	}

	// if V is exist
	if (m_topV.find(vol) != m_topV.end())
		return false;

	// if T & V are not exist
	m_topV.insert(make_pair(vol, key));
	m_topT.insert(make_pair(key, vol));

	while (m_topV.size() > m_num)
	{
		auto it = m_topV.begin();
		string dkey = it->second;
		auto it2 = m_topT.find(dkey);
		if (it2 != m_topT.end())
		{
			m_topT.erase(it2);
			m_topV.erase(it);
		}
	}
	return true;
}

bool TopVolume::is_top(int vol)
{
	if (this->m_topV.empty())
		return false;

	auto x = this->m_topV.begin();
	if (x->first <= vol)
		return true;
	else
		return false;
}

void TopVolume::clear()
{
	m_topT.clear();
	m_topV.clear();
}

void TopVolume::print()
{
	if (this->m_topV.empty())
		cout << "TopVolume is empty!" << endl;
	else
	{
		cout << "TopVolume " << this->m_num << " as :\n";
		for (map<string, int>::iterator it = m_topT.begin(); it != m_topT.end(); ++it)
		{
			cout << it->first << "0,  " << it->second << "\n";
		}
		cout << endl;
	}
}
