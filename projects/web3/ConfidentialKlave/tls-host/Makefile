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

NAME := tee-wallet-service
ARCH ?= aarch64

OPTEE_DIR ?= ../../../optee

ifeq ($(ARCH), arm)
	OPTEE_BIN := $(OPTEE_DIR)/toolchains/aarch32/bin
	OBJCOPY := $(OPTEE_BIN)/arm-linux-gnueabihf-objcopy
	TARGET := arm-unknown-linux-gnueabihf
else
	OPTEE_BIN := $(OPTEE_DIR)/toolchains/$(ARCH)/bin
	OBJCOPY := $(OPTEE_BIN)/aarch64-linux-gnu-objcopy
	TARGET := aarch64-unknown-linux-gnu
endif

OUT_DIR := $(CURDIR)/target/$(TARGET)/release


all: host strip

host:
	@cargo build --target $(TARGET) --release

strip:
	@$(OBJCOPY) --strip-unneeded $(OUT_DIR)/$(NAME) $(OUT_DIR)/$(NAME)

clean:
	@cargo clean
