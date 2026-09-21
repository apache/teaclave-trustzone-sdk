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

TARGET ?= $(or $(TARGET_HOST),aarch64-unknown-linux-gnu)
ifneq ($(filter arm-unknown-%,$(TARGET)),)
OPTEE_ARCH := --arch arm
else
OPTEE_ARCH := --arch aarch64
endif
ifneq ($(CROSS_COMPILE_HOST),)
CROSS_COMPILE := $(CROSS_COMPILE_HOST)
endif
ifneq ($(CROSS_COMPILE),)
export CROSS_COMPILE
endif
SDK_ROOT := $(abspath $(CURDIR)/../../../..)
CARGO_OPTEE ?= cargo run --manifest-path $(SDK_ROOT)/tools/Cargo.toml -q -p cargo-optee --

.PHONY: all clippy host strip clean

all: host

clippy:
	@$(CARGO_OPTEE) clippy ca $(OPTEE_ARCH)

host: clippy
	@$(CARGO_OPTEE) build ca $(OPTEE_ARCH)

strip: host

clean:
	@cargo clean
