
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

# 基于SFML实现图形化

line在sfml中如同Rectangle一样，本身就是一种Shape，直接支持draw，如下：

 sf::Vertex line1[] =
   {
        sf::Vertex(sf::Vector2f(10, 10), sf::Color(255,0,0)),
        sf::Vertex(sf::Vector2f(20, 56), sf::Color(255,0,0))
   };

  window.clear();
  window.draw(line, 2, sf::Lines);
  // 为啥第二个参数是2,  线的两端，实际给个3也可以，但没有用！ 当第三个参数为sf::LinesStrip, 第二个参数就很有用了！！！


使用sf::Lines进行连续曲线：

sf::Vertex m_Lines[lines]; 
for (int i=0; i< lines; i++){
            float x = 10 + i*1;
            float y = (float)(300+ (i%2+1)*20);
            m_Lines[i] = sf::Vertex(sf::Vector2f(x, y), sf::Color(255,0,0));
} 

… …

window.clear();
for (int i=0; i< lines-1; i++){
            window.draw(&m_Lines[i], 2, sf::Lines);
}
window.display();
 

使用sf::PrimitiveType::LineStrip)

通过LineStrip可以简化连续的画线：

window.clear();
window.draw(m_Lines, lines, sf::PrimitiveType::LineStrip);
 
# ta-lib支持

wget http://prdownloads.sourceforge.net/ta-lib/ta-lib-0.4.0-src.tar.gz

tar xvzf ta-lib-0.4.0-src.tar.gz
cd ta-lib
./configure 
make 
sudo make install

注意安装路径
include: /usr/local/include/ta-lib
lib: /usr/local/lib

简要说明
ta-lib的函数定义都在ta_func.h中
函数通常用TA_XXXX命名，如：TA_RetCode TA_BBANDS(… …)
调用前需要初始化（只需启动一次）：TA_Initialize()
调用后需要释放资源（只需退出前调用一次）：TA_Shutdown();
返回的两个int型数据特别注意：outBeg, outNbElement, 前者是指Out数据与In数据的第几项开始对齐，后者是指Out数据的有效长度。 
 
example:
#include <iostream>
#include <ta-lib/ta_libc.h>
#include <ta-lib/ta_func.h>
using namespace std;
int main(){

	printf("try to initialize");
	//Technical analysis part of the code
	TA_Real closePrice[40];
	TA_Real out[40]; 
	TA_Integer outBeg; 
	TA_Integer outNbElement; 
	/* ... initializeyour closing price here... */ 
	for (int i=0; i<40; i++){
		closePrice[i]=1.0*(i%8);
	}
	
	TA_RetCode retCode;
	retCode = TA_Initialize();
	if( retCode != TA_SUCCESS )
		printf("Cannot initialize TA-Lib !");
	else
	{
		printf("TA-Lib correctly initialized.\n") ;
		retCode = TA_MA(0, 39, &closePrice[0], 30, TA_MAType_SMA, &outBeg, &outNbElement, &out[0] ); 
		for(int i=0; i <outNbElement; i++ ) 
			 printf("Day %d = %f\n", outBeg+i, out[i] ); 
		TA_Shutdown();
	}
	return 0;
}

## Makefile
cc = g++
app = abc
deps = $(shell find ./ -name "*.h")
src = $(shell find ./ -name "*.cc")
obj = $(src:%.cc=%.o)

app: $(obj)
    $(cc) -o $(app) $(obj) -fPIC -L/usr/local/lib -lta_lib 
%.o: %.cc $(deps)
    $(cc) -c $< -I/usr/local/include 

clean:
    rm -rf $(obj) $(app)
 

## 可能的编译错误：
g++ -c abc.cc -o abc.o -I/usr/local/include -L/usr/local/lib 
g++ -o  ./abc.o  -std=c++17 -lta_lib 
/usr/bin/ld: /usr/lib/gcc/x86_64-linux-gnu/12/../../../x86_64-linux-gnu/Scrt1.o: in function `_start':
(.text+0x1b): undefined reference to `main'
collect2: error: ld returned 1 exit status

动态库链接编译，增加参数 -fPIC

## 关于ta-lib的函数

可通过 ta-lib/c/include/ta_func.h 中定义的接口直接调用。

所有 TA 函数都是简单的数学函数。提供一个数组作为输入，函数只需将输出存储在调用者提供的输出数组中。TA 函数不会为调用者分配任何空间。输出中的数据数永远不会超过请求计算的元素数（下文将解释 startIdx 和 endIdx）。

# for Macos





