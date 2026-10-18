.PHONY: all bpf clean bench

BPF_SRC = bpf/xdp_redirect.c
BPF_OBJ = bpf/xdp_redirect.o
CLANG ?= clang
LLVM_STRIP ?= llvm-strip
BPFTOOL ?= bpftool

CFLAGS = -O2 -g -target bpf
ARCH := $(shell uname -m | sed 's/x86_64/x86/' | sed 's/aarch64/arm64/')

all: bpf

bpf: $(BPF_OBJ)

$(BPF_OBJ): $(BPF_SRC)
	$(CLANG) $(CFLAGS) -D__TARGET_ARCH_$(ARCH) -c $< -o $@
	$(LLVM_STRIP) -g $@

check: $(BPF_OBJ)
	$(BPFTOOL) prog show $<

clean:
	rm -f $(BPF_OBJ)

bench:
	cargo bench --bench throughput

run:
	sudo cargo run --release

attach:
	sudo ip link set dev $(IFACE) xdp obj $(BPF_OBJ) sec xdp

detach:
	sudo ip link set dev $(IFACE) xdp off
