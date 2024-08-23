TARGET := riscv64gc-unknown-none-elf
MODE := debug

ifeq ($(BOARD), 2k1000)
	TARGET := loongarch64-unknown-linux-gnu
endif
ifeq ($(BOARD), laqemu)
	TARGET := loongarch64-unknown-linux-gnu
endif


ARCH=${shell echo ${TARGET}|cut -d- -f1 |grep -o "^[a-zA-Z]\+[[:digit:]]*"}
ifeq ($(ARCH), loongarch64)
	BOARD ?= laqemu
endif

build: rust-user

env:
	-(rustup target list | grep "$(TARGET) (installed)") || rustup target add $(TARGET)
	-cargo install --list |grep cargo-binutils |grep "v0.2.0">/dev/null || cargo install cargo-binutils --vers ~0.2
	-rustup component add rust-src
	-rustup component add llvm-tools-preview

rust-user: env $(APPS)
ifeq ($(MODE),release)
	@cargo build --target=$(TARGET) --release
else
	@cargo build --target=$(TARGET)
endif

clean:
	@cargo clean
	@cd ${TESTDIR} && make clean

.PHONY: build rust-user clean
