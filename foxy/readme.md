# 期货（CTP）程序开发的几个关键要素
## 1. 配置信息
 - 订阅服务 md_front="tcp://121.37.80.177:20004"
 - 交易服务 td_front = “tcp://121.37.80.177:20002”
 - user = 'xxx'
 - password = 'xxx'
 - broker_id = ''
 - authcode = ''
 - appid = ''

查看服务器地址：http://121.37.80.177:50080/detail.html

## 2. 动态库
 - TTS动态库：   libthostmduserapi_se.so  libthosttraderapi_se.so
 - CTP动态库：   thostmduserapi_se.so  thosttraderapi_se.so

## 3. 获取争取的API及lib

### CTPAPI+lib
https://github.com/openctp/openctp/tree/master/docs/CTPAPI
或者官网（非交易时间禁止访问）：
http://www.sfit.com.cn
https://www.simnow.com.cn/

### OpenCTP TTS lib
https://github.com/openctp/openctp-tts-python

# 基于 OpenCTP TTS

 - TTS系统是一个集股票（A股、港股、美股）、期货、期权于一体的综合交易系统，与CTP系统架构类似，采用内存数据库架构。
 - OpenCTP搭建了一个完全模拟Simnow的环境，基于TTS，采用完全兼容的接口规范

7x24环境：
交易前置：tcp://121.37.80.177:20002
行情前置：tcp://121.37.80.177:20004

仿真环境：交易时段同实盘，其它时间也可交易，只是价格不再变化
交易前置：tcp://121.37.90.193:20002

## 灵魂3问：
 1. 为何选择 OpenCTP TTS，因为可以24小时调试程序，完成后需要切换到CTP，则只要替换掉2个lib库重新编译即可！
 2. 从哪里获取 OpenCTP TTS lib？ https://github.com/openctp/openctp-tts-python/openctp-tts-python-main 下选择对应版本目录下的对应系统，如6.6.7/linux64/，保护了头文件和.so文件。
 3. 7大配置信息如何获得？
md/td获取：http://121.37.80.177:50080/detail.html
 - OpenCTP 订阅不需要申请账号，交易需要申请。
关注openctp公众号，即可获得一个7x24账号及一个仿真账号，也可以回复"注册24"或"注册仿真"再申请新的模拟号。
openctp除免费仿真环境外，还提供了vip仿真环境。
vip权限申请方法：关注openctp公众号并回复“注册vip”。
 - SimNow需要申请：
官网（非交易时间禁止访问）：http://www.simnow.com.cn
交易者注册仿真账户后，可以使用从CTP官网下载的API接入这套仿真交易系统。开发、测试完成之后，只需要更换账户密码、前置地址等信息就可以接入期货公司生产系统进行实盘交易。
```text
user（investorId）：218695
password: x94N2ewEHq!
brokerId：9999
broker_id = '9999'
authcode = '0000000000000000'
appid = 'simnow_client_test'
```


# 重要的编译参数：动态库

makefile中加入目标动态库（当前路径下，tts版本）：
libthostmduserapi_se.so 
libthosttraderapi_se.so


# 基于CTP开发

与TTS重要的区别就是需要依赖Simnow环境（经常停机）！，主要2个方面需要注意：
 - 账号和服务器地址信息不同；
 - 编译打包的动态库不同；

## 申请SimNow账号
官网（非交易时间禁止访问）：http://www.simnow.com.cn
开发、测试完成之后，只需要更换账户密码、前置地址等信息就可以接入期货公司生产系统进行实盘交易。

服务停机是家常便饭，需要即使查看：http://121.37.80.177:50080/detail.html


```text
user（investorId）：218695
password: x94N2ewEHq!
brokerId：9999
broker_id = '9999'
authcode = '0000000000000000'
appid = 'simnow_client_test'
```

# 重要的编译参数：动态库
makefile中加入目标动态库（当前路径下，ctp版本）：
thostmduserapi_se.so 
thosttraderapi_se.so


# install libcurl for httpclient

download libcurl from https://curl.se/download/curl-8.4.0.zip
unzip and cd curl path
./configure
make
sudo make install
edit makefile, add -lcurl
if can't find libcurl.so, then
将部署目录路径(如/usr/local/lib） 写入 /etc/ld.so.conf 文件最后一行，并执行 /sbin/ldconfig 命令

# 安装Crow,提供REST RPC服务

download src from https://github.com/CrowCpp/Crow
[备选] https://gitcode.com/mirrors/ipkn/crow
cp -R include /usr/include/crow

```shell
#Ubuntu
sudo apt-get install build-essential libtcmalloc-minimal4 && sudo ln -s /usr/lib/libtcmalloc_minimal.so.4 /usr/lib/libtcmalloc_minimal.so
#OSX
brew install boost google-perftools
```
由于boost库在1.7以后的版本移除了get_io_service。 
vim include/crow/socket_adaptors.h，做如下修改：
```c++
 boost::asio::io_service& get_io_service()
 {
     // return socket_.get_io_service();
     return (boost::asio::io_context&)(socket_).get_executor().context();
 }
```





