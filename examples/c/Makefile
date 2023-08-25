CC ?= clang
TARGET_DIR = ../../target/x86_64-unknown-linux-musl/release

.PHONY: all clean
all: clean load_unload

load_unload:
	$(CC) -o $@ $@.c $(TARGET_DIR)/libpmem.a

clean:
	rm load_unload
