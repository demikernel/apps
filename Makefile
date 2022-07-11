# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

export PREFIX ?= $(HOME)

#===============================================================================

export CARGO ?= $(HOME)/.cargo/bin/cargo
export PKG_CONFIG_PATH ?= $(shell find $(PREFIX)/lib/ -name '*pkgconfig*' -type d | xargs | sed -E 's/ /:/g')
export LD_LIBRARY_PATH ?= $(shell find $(PREFIX)/lib/ -name '*x86_64-linux-gnu*' -type d | xargs | sed -E 's/ /:/g')
export CONFIG_PATH ?= $(HOME)/config.yaml

#===============================================================================

export SRCDIR = $(CURDIR)/src

#===============================================================================

export DRIVER ?= $(shell  [ ! -z "`lspci | grep -E "ConnectX-[4,5]"`" ] && echo mlx5 || echo mlx4)
export BUILD ?= --release
export MSS ?= 1500
export MTU ?= 1500
export LOCAL ?= 127.0.0.1:12345
export REMOTE ?= 127.0.0.1:23456
export LIBOS ?= catnap
export BUFSIZE ?= 64
export INJECTION_RATE ?= 1000
export TIMEOUT ?= 180

#===============================================================================

all:
	$(CARGO) build --all $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER)

run-tcp-dump:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin tcp-dump -- --local $(LOCAL)

run-tcp-echo-server:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin tcp-echo -- --peer server --local $(LOCAL) --bufsize=$(BUFSIZE)

run-tcp-echo-client:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin tcp-echo -- --peer client --remote $(REMOTE) --bufsize=$(BUFSIZE)

run-tcp-pktgen:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin tcp-pktgen -- --remote $(REMOTE) --bufsize=$(BUFSIZE) --injection_rate=$(INJECTION_RATE)

run-udp-dump:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin udp-dump -- --local $(LOCAL)

run-udp-echo:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin udp-echo -- --local $(LOCAL) --remote $(REMOTE)

run-udp-pktgen:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin udp-pktgen -- --local $(LOCAL) --remote $(REMOTE) --bufsize=$(BUFSIZE) --injection_rate=$(INJECTION_RATE)

run-udp-relay:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin udp-relay -- --local $(LOCAL) --remote $(REMOTE)

clean:
	rm -rf target && \
	$(CARGO) clean && \
	rm -f Cargo.lock
