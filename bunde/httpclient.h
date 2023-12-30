
#include <iostream>
#include <curl/curl.h>
#include <string>
#include <stdlib.h>
using namespace std;

class HttpClient {
public:
    HttpClient() {
        curl_global_init(CURL_GLOBAL_ALL);
        curl_ = curl_easy_init();
    }

    ~HttpClient() {
        curl_easy_cleanup(curl_);
    }

    bool Post(const std::string& url, const std::string& data, const string &header, std::string& response) {
            if (!curl_) {
                    return false;
            }
            // set params
            // set curl header
            struct curl_slist* header_list = NULL;
            header_list = curl_slist_append(header_list, "User-Agent: Mozilla/5.0 (Windows NT 10.0; WOW64; Trident/7.0; rv:11.0) like Gecko");
            header_list = curl_slist_append(header_list, "Content-Type:application/json");
	    if (header.length() > 0)
	            header_list = curl_slist_append(header_list, header.c_str());

	    curl_easy_setopt(curl_, CURLOPT_HTTPHEADER, header_list);

            curl_easy_setopt(curl_, CURLOPT_URL, url.c_str());
            curl_easy_setopt(curl_, CURLOPT_POST, 1L);
            curl_easy_setopt(curl_, CURLOPT_POSTFIELDS, data.c_str());
            curl_easy_setopt(curl_, CURLOPT_WRITEFUNCTION, &WriteCallback);
            curl_easy_setopt(curl_, CURLOPT_WRITEDATA, &response);

            CURLcode res = curl_easy_perform(curl_);
            return (res == CURLE_OK);
    }

    bool Get(const string& url, string& response) {
        if (!curl_) {
            return false;
        }

        curl_easy_setopt(curl_, CURLOPT_URL, url.c_str());
                curl_easy_setopt(curl_, CURLOPT_POST, 0L);
        curl_easy_setopt(curl_, CURLOPT_WRITEFUNCTION, &WriteCallback);
        curl_easy_setopt(curl_, CURLOPT_WRITEDATA, &response);

        CURLcode res = curl_easy_perform(curl_);
        return (res == CURLE_OK);
    }

private:
    CURL* curl_ = nullptr;

    static size_t WriteCallback(void* contents, size_t size, size_t nmemb, void* userp) {
        size_t realsize = size * nmemb;
        std::string* str = static_cast<std::string*>(userp);
        str->append(static_cast<char*>(contents), realsize);
        return realsize;
    }
};

void macd_warnning_to_notify(string title, float vprice, string remote_svr ="" ){
	HttpClient hc;
	string resp;
	if (remote_svr.length() < 10)
                return;
        
        //remote_svr = "http://localhost:9999/xman/sendmessage";
	string s;
	s.append(title);
	s.append(", 价格拐点来临，落差:");
	s.append(to_string(vprice));
	string json = "{\"recv_id\":\"oc_208983102250542d93cb4081935fc489\",\"content\":\"";
        json.append(s);
	json.append("\",\"recv_type\":\"chat_id\", \"msg_type\":\"text\"}");
        hc.Post( remote_svr, json,"Authorization:Bearer xman",  resp);
}

string read_ticks_from_server(string remote_svr, string name, string dtime ){
	HttpClient hc;
	string resp;
	string remote_api = remote_svr; 
        if (remote_api.length() < 10)
                remote_api = "http://localhost:9996/bunde?";
	remote_api.append("/bunde?name=");
	remote_api.append(name);
	remote_api.append("&dtime=");
	remote_api.append(dtime);
        hc.Get(remote_api, resp);
	cout << "read from bunny:" << remote_api << endl;
	return resp;
}

/*
int main(){

        HttpConnection ht;
        string resp;
        // ht.Post("http://192.168.2.108:9996/newfund","", &resp);
        if (ht.Get("http://192.168.2.108:9996", resp)){
                cout << resp << endl;
        }
        string json("{\"TradingDay\":\"20231013\",\"InstrumentID\":\"AP401\",\"LastPrice\":9379,\"PreSettlementPrice\":9326,\"PreClosePrice\":9373,\"PreOpenInterest\":123856,\"OpenPrice\":9389,\"Volume\":54654,\"Turnover\":511397478,\"OpenInterest\":128462,\"ClosePrice\":1.7976931348623157e+308,\"UpdateTime\":\"23:32:26\",\"AveragePrice\":9357}");
        if(ht.Post("http://192.168.2.108:9996/newfund", json , resp)){
                cout<<resp<<endl;
        }

        return 0;
}
*/
