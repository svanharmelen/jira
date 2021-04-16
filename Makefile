release:
	cargo build --release

linux:
	CC_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-gcc CMAKE_TOOLCHAIN_FILE=$(shell pwd)/linux-gnu-x86_64.cmake cargo build --target=x86_64-unknown-linux-gnu --release
