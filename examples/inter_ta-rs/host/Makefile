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

NAME := inter_ta-rs

TARGET ?= aarch64-unknown-linux-gnu
CROSS_COMPILE ?= aarch64-linux-gnu-
OBJCOPY := $(CROSS_COMPILE)objcopy
LINKER_CFG := target.$(TARGET).linker=\"$(CROSS_COMPILE)gcc\"

OUT_DIR := $(CURDIR)/target/$(TARGET)/release

all: clippy host strip

clippy:
	@cargo fmt
	@cargo clippy --target $(TARGET_HOST) -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic

host: clippy
	@cargo build --target $(TARGET_HOST) --release --config $(LINKER_CFG)

strip: host
	@$(OBJCOPY) --strip-unneeded $(OUT_DIR)/$(NAME) $(OUT_DIR)/$(NAME)

clean:
	@cargo clean
