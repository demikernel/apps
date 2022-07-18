# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.

export PREFIX ?= $(HOME)

#===============================================================================

export CARGO ?= $(HOME)/.cargo/bin/cargo
export PKG_CONFIG_PATH ?= $(shell find $(PREFIX)/lib/ -name '*pkgconfig*' -type d | xargs | sed -E 's/ /:/g')
export LD_LIBRARY_PATH ?= $(shell find $(PREFIX)/lib/ -name '*x86_64-linux-gnu*' -type d | xargs | sed -E 's/ /:/g')
export CONFIG_PATH ?= $(HOME)/config.yaml

#=======================================================================================================================
# Build Parameters
#=======================================================================================================================

export LIBOS ?= catnap
export CARGO_FEATURES := --features=$(LIBOS)-libos

# Switch for DPDK
ifeq ($(LIBOS),catnip)
DRIVER ?= $(shell [ ! -z "`lspci | grep -E "ConnectX-[4,5]"`" ] && echo mlx5 || echo mlx4)
CARGO_FEATURES += --features=$(DRIVER)
endif

CARGO_FEATURES += $(FEATURES)

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
export TIMEOUT ?= 180

#===============================================================================

all:
	$(CARGO) build --all $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS)

run-tcp-dump:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS) --bin tcp-dump -- --local $(LOCAL)

run-tcp-echo-server:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS) --bin tcp-echo -- --peer server --local $(LOCAL) --bufsize=$(BUFSIZE)

run-tcp-echo-client:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS) --bin tcp-echo -- --peer client --remote $(REMOTE) --bufsize=$(BUFSIZE)

run-tcp-pktgen:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS) --bin tcp-pktgen -- --remote $(REMOTE) --bufsize=$(BUFSIZE) --injection_rate=$(INJECTION_RATE)

run-udp-dump:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS) --bin udp-dump -- --local $(LOCAL)

run-udp-echo:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS) --bin udp-echo -- --local $(LOCAL) --remote $(REMOTE)

run-udp-pktgen:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS) --bin udp-pktgen -- --local $(LOCAL) --remote $(REMOTE) --bufsize=$(BUFSIZE) --injection_rate=$(INJECTION_RATE)

run-udp-relay:
	timeout $(TIMEOUT) $(CARGO) run $(BUILD) $(CARGO_FEATURES) $(CARGO_FLAGS) --bin udp-relay -- --local $(LOCAL) --remote $(REMOTE)

# Check code style formatting.
check-fmt: check-fmt-rust

# Check code style formatting for Rust.
check-fmt-rust:
	$(CARGO) fmt --all -- --check

# Builds documentation.
doc:
	$(CARGO) doc $(CARGO_FEATURES) $(CARGO_FLAGS) --no-deps

clean:
	rm -rf target && \
	$(CARGO) clean && \
	rm -f Cargo.lock
