动态数据的趋势图

最初的愿望是用rust开发一个期货行情+策略的实时曲线跟踪软件。

最初采用了ggez和bevy，因为这种游戏引擎对于图像元素的输出非常适合。但实现后发现这些系统处于游戏支持的原因，考虑的东西太多太复杂，CPU占用率高，渲染能耗太大，内存占用过大。
以ggez为例，一个期货跟踪曲线，在我的电脑上(windows10)，ggez占用了3~8%的CPU,10%的GPU，耗电等级为低（降低更新频率情况下），内存占用达到460M。
为此，非常需要一款低能耗的解决方案。

# 目标

- 期货行情跟踪与策略分析
- 实现一个高性能的动态数据可视化库，少占用CPU资源，少占用内存资源，少占用GPU资源，少消耗电能。
- 具有python plot类似得能力，又支持实时动态显示。


# 目录及项目介绍

## foxy
c++基于CTP开发的行情订阅转发器，解耦软件对于CTP行情的关系。foxy通过UDP发出行情信息，是下面其他软件的基础。

## bunde
用c++实现的基于SFML的期货行情跟踪及策略分析。

## cunder
用rust基于ggez开发的期货行情跟踪及策略分析。

## fare
用rust基于plotters+egui开发的期货行情跟踪及策略分析。

### 特点

- 基于plotters/plotters_backend + egui/eframe 
- 基于rust
- 基于动态数据,动态曲线
- 异步多线程，upd监听接收行情
- csv保存或读取数据
- 1%以内的CPU资源，60M的内存资源，极少的电力消耗
  

# 涉及的rust库

## plotters

[plotters docs](https://docs.rs/plotters/latest/plotters/)

[plotters source](https://github.com/plotters-rs/plotters)

Plotters 是一个绘图库，用于以纯 Rust 语言渲染数字、绘图和图表。Plotters 支持各种类型的后端，包括位图、矢量图、活塞窗口、GTK/Cairo 和 WebAssembly。
- 使用交互式 Jupyter 笔记本试用 Plotters，或点击此处查看静态 HTML 版本。
- 目前，我们已为控制台绘图准备好所有内部代码，但基于控制台的后台仍未就绪。有关如何使用自定义后端在控制台上绘图，请参阅此示例。

 
## egui

[egui docs](https://docs.rs/egui/latest/egui/)

[egui source](https://github.com/emilk/egui)

是一个简单、快速、高度可移植的 Rust 即时模式图形用户界面库。
egui 的目标是成为最易用的 Rust 图形用户界面库，以及用 Rust 制作网络应用程序的最简单方法。
egui 可以在任何可以绘制纹理三角形的地方使用，这意味着你可以轻松地将它集成到你选择的游戏引擎中。

### These are the official egui integrations:

- eframe for compiling the same app to web/wasm and desktop/native. Uses egui-winit and egui_glow or egui-wgpu.
- egui_glow for rendering egui with glow on native and web, and for making native apps.
- egui-wgpu for wgpu (WebGPU API).
- egui-winit for integrating with winit.

### eframe

eframe 是使用 egui 编写应用程序的官方框架库。应用程序既可编译为本地运行（跨平台），也可编译为网络应用（使用 WASM）。
eframe uses egui_glow for rendering, and on native it uses egui-winit.

### egui/eframe on linux

To use on Linux, first run:
```text
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```


## ggez

 ggez是一个rust轻量级的2D游戏图形引擎，它的目标是让游戏开发尽量的简单。https://github.com/ggez/ggez

## SFML

SFML是一个跨平台的c++多媒体库，可以简化游戏和多媒体应用程序的开发。
https://github.com/SFML/SFML


## CTP

https://github.com/openctp/openctp/tree/master/docs/CTPAPI 
或者官网（非交易时间禁止访问）： http://www.sfit.com.cn https://www.simnow.com.cn/

## Crow
借助Crow，提供REST RPC服务，可以开发交易等相关功能的解耦。

download src from https://github.com/CrowCpp/Crow [备选] https://gitcode.com/mirrors/ipkn/crow cp -R include /usr/include/crow
```text
#Ubuntu
sudo apt-get install build-essential libtcmalloc-minimal4 && sudo ln -s /usr/lib/libtcmalloc_minimal.so.4 /usr/lib/libtcmalloc_minimal.so
#OSX
brew install boost google-perftools
```
由于boost库在1.7以后的版本移除了get_io_service。 vim include/crow/socket_adaptors.h，做如下修改：
```cpp
 boost::asio::io_service& get_io_service()
 {
     // return socket_.get_io_service();
     return (boost::asio::io_context&)(socket_).get_executor().context();
 }
```
