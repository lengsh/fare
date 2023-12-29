cc = g++
tts = foxytts
ctp = foxy
deps = $(shell find ./ -name "*.h")
src = $(shell find ./ -name "*.cc")
obj = $(src:%.cc=%.o) 

ctp: $(obj)
	$(cc) -o $(ctp) $(obj) -lcurl thostmduserapi_se.so thosttraderapi_se.so -std=c++17

tts: $(obj)
	$(cc) -o $(tts) $(obj) -lcurl libthostmduserapi_se.so libthosttraderapi_se.so -std=c++17

%.o: %.cc $(deps)
	$(cc) -c $< -o $@ -I./api -I/usr/include/crow

clean:
	rm -rf $(obj) $(ctp) $(tts)
	rm ./*.con	
