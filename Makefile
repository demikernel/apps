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
export BUFSIZE ?= 1024
export INJECTION_RATE ?= 1000

#===============================================================================

all:
	echo $(PKG_CONFIG_PATH)
	$(CARGO) build --all $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER)

run-udp-dump:
	$(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin udp-dump -- --local $(LOCAL)

run-udp-echo:
	$(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin udp-echo -- --local $(LOCAL) --remote $(REMOTE)

run-udp-pktgen:
	$(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin udp-pktgen -- --local $(LOCAL) --remote $(REMOTE) --bufsize=$(BUFSIZE) --injection_rate=$(INJECTION_RATE)

run-udp-relay:
	$(CARGO) run $(BUILD) $(CARGO_FLAGS) --features=$(LIBOS)-libos --features=$(DRIVER) --bin udp-relay -- --local $(LOCAL) --remote $(REMOTE)

clean:
	rm -rf target && \
	$(CARGO) clean && \
	rm -f Cargo.lock
