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

# STD-ONLY example

UUID ?= $(shell cat "../uuid.txt")

TARGET ?= aarch64-unknown-linux-gnu
CROSS_COMPILE ?= aarch64-linux-gnu-
OBJCOPY := $(CROSS_COMPILE)objcopy
# Configure the linker to use GCC, which works on both cross-compilation and ARM machines
LINKER_CFG := target.$(TARGET).linker=\"$(CROSS_COMPILE)gcc\"

# fix for the error: "unwinding panics are not supported without std" reported by clippy
# Set panic=abort for std and no-std
RUSTFLAGS := -C panic=abort

TA_SIGN_KEY ?= $(TA_DEV_KIT_DIR)/keys/default_ta.pem
SIGN := $(TA_DEV_KIT_DIR)/scripts/sign_encrypt.py
OUT_DIR := $(CURDIR)/target/$(TARGET)/release

BUILDER ?= cargo
FEATURES ?= 

all: clippy ta strip sign

clippy:
	@cargo fmt
	@RUSTFLAGS="$(RUSTFLAGS)" $(BUILDER) clippy --target $(TARGET) $(FEATURES) -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic

ta: clippy
	@RUSTFLAGS="$(RUSTFLAGS)" $(BUILDER) build --target $(TARGET) --release $(FEATURES) --config $(LINKER_CFG)

strip: ta
	@$(OBJCOPY) --strip-unneeded $(OUT_DIR)/ta $(OUT_DIR)/stripped_ta

sign: strip
	@$(SIGN) --uuid $(UUID) --key $(TA_SIGN_KEY) --in $(OUT_DIR)/stripped_ta --out $(OUT_DIR)/$(UUID).ta
	@echo "SIGN =>  ${UUID}"

clean:
	@cargo clean
