
# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

# Makefile for all host (CA) applications and plugins

# Use the package name from Cargo.toml for binary name
OUT_DIR := $(CURDIR)/target/$(TARGET_HOST)/release
OBJCOPY := $(CROSS_COMPILE_HOST)objcopy
LINKER_CFG := target.$(TARGET_HOST).linker=\"$(CROSS_COMPILE_HOST)gcc\"

# Binary name defined in Cargo.toml
CARGO_BINARY_NAME := $(strip $(shell grep '^name *=' Cargo.toml | head -n1 | sed 's/.*= *"\([^"]*\)".*/\1/'))

# Plugin specific variables (ignored if not a plugin)
IS_PLUGIN ?= false
PLUGIN_UUID ?= $(shell cat ../plugin_uuid.txt)

CLIPPY_OPTS := -D warnings \
               -D clippy::unwrap_used \
               -D clippy::expect_used \
               -D clippy::panic

.PHONY: all clippy build post_build clean emulate

all: clippy build post_build

clippy:
	@cargo fmt
	@cargo clippy --target $(TARGET_HOST) -- $(CLIPPY_OPTS)

build: clippy
	@cargo build --target $(TARGET_HOST) --release --config $(LINKER_CFG)

# Handle different post-build steps for plugins vs normal binaries
post_build: build
ifeq ($(IS_PLUGIN),true)
	cp $(OUT_DIR)/lib$(CARGO_BINARY_NAME).so $(OUT_DIR)/$(PLUGIN_UUID).plugin.so
else
	@$(OBJCOPY) --strip-unneeded $(OUT_DIR)/$(CARGO_BINARY_NAME) $(OUT_DIR)/$(CARGO_BINARY_NAME).stripped.elf
endif

emulate: all
	@sync_to_emulator --host $(OUT_DIR)/$(CARGO_BINARY_NAME)

clean:
	@cargo clean
