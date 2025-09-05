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

OPTEE_DIR ?= ../../../optee
OPTEE_OS_DIR ?= $(OPTEE_DIR)/optee_os
UUID ?= $(shell cat "../tls_uuid.txt")

ARCH ?= aarch64

ifeq ($(ARCH), arm)
	TA_SIGN_KEY ?= $(OPTEE_OS_DIR)/out/arm/export-ta_arm32/keys/default_ta.pem
	SIGN := $(OPTEE_OS_DIR)/out/arm/export-ta_arm32/scripts/sign_encrypt.py
	OPTEE_BIN := $(OPTEE_DIR)/toolchains/aarch32/bin
	OBJCOPY := $(OPTEE_BIN)/arm-linux-gnueabihf-objcopy
	TARGET := arm-unknown-optee-trustzone
else
	TA_SIGN_KEY ?= $(OPTEE_OS_DIR)/out/arm/export-ta_arm64/keys/default_ta.pem
	SIGN := $(OPTEE_OS_DIR)/out/arm/export-ta_arm64/scripts/sign_encrypt.py
	OPTEE_BIN := $(OPTEE_DIR)/toolchains/$(ARCH)/bin
	OBJCOPY := $(OPTEE_BIN)/aarch64-linux-gnu-objcopy
	TARGET := aarch64-unknown-optee-trustzone
endif

OUT_DIR := $(CURDIR)/target/$(TARGET)/release
BIN_NAME := $(shell cargo metadata --no-deps --format-version 1 | grep -oP '(?<="name":")[^"]+' | sed -n 1p)

all: fmt ta strip sign

ta:
	@xargo build --target $(TARGET) --release --verbose

strip:
	@$(OBJCOPY) --strip-unneeded $(OUT_DIR)/$(BIN_NAME) $(OUT_DIR)/stripped_ta

sign:
	@$(SIGN) --uuid $(UUID) --key $(TA_SIGN_KEY) --in $(OUT_DIR)/stripped_ta --out $(OUT_DIR)/$(UUID).ta
	@echo "SIGN =>  ${UUID}"

fmt: 
	@cargo fmt

clippy: 
	@xargo clippy --target $(TARGET) --release --verbose

clean:
	@xargo clean
