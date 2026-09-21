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

ifeq ($(O),)
out-dir := $(CURDIR)/out
else
out-dir := $(O)
endif

bindir ?= /usr/bin
libdir ?= /usr/lib

ifneq ($V,1)
	q := @
	echo := @echo
else
	q :=
	echo := @:
endif
# export 'q', used by sub-makefiles.
export q

TARGET ?= aarch64-unknown-linux-gnu

# If _HOST or _TA specific compiler/target are not specified, then use common
# compiler/target for both. OP-TEE rust.mk still passes TARGET_* / CROSS_COMPILE_*.
# Do not default CROSS_COMPILE_*; cargo-optee derives the prefix from --arch.
CROSS_COMPILE_HOST ?= $(CROSS_COMPILE)
CROSS_COMPILE_TA ?= $(CROSS_COMPILE)
TARGET_HOST ?= $(TARGET)
TARGET_TA ?= $(TARGET)

.PHONY: all examples std-examples no-std-examples \
	install clean examples-clean help

ifneq ($(wildcard $(TA_DEV_KIT_DIR)/host_include/conf.mk),)
all: examples
else
all:
	$(q)echo "TA_DEV_KIT_DIR is not correctly defined" && false
endif

# Default examples target - builds no-std examples for backward compatibility
examples: no-std-examples

# Delegate all examples-related targets to examples/Makefile
std-examples no-std-examples install:
	$(q)$(MAKE) -C examples $@ \
		TA_INSTALL_DIR="$(abspath $(out-dir))/lib/optee_armtz" \
		CA_INSTALL_DIR="$(abspath $(out-dir))$(bindir)" \
		PLUGIN_INSTALL_DIR="$(abspath $(out-dir))$(libdir)/tee-supplicant/plugins" \
		$(if $(STD),STD="$(STD)") \
		TARGET_HOST=$(TARGET_HOST) \
		TARGET_TA=$(TARGET_TA) \
		CROSS_COMPILE_HOST=$(CROSS_COMPILE_HOST) \
		CROSS_COMPILE_TA=$(CROSS_COMPILE_TA) \
		TA_DEV_KIT_DIR=$(TA_DEV_KIT_DIR) \
		OPTEE_CLIENT_EXPORT=$(OPTEE_CLIENT_EXPORT) \
		$(if $(TA_SIGN_KEY),TA_SIGN_KEY="$(TA_SIGN_KEY)")

clean: examples-clean out-clean

examples-clean:
	$(q)$(MAKE) -C examples clean

out-clean:
	rm -rf out

help:
	@echo "Available targets:"
	@echo "  examples              - Build no-std examples (default, backward compatible)"
	@echo "  std-examples          - Build std examples (std-only + common)"
	@echo "  no-std-examples       - Build no-std examples (no-std-only + common)"
	@echo "  install               - Build and install examples to out directory"
	@echo "  clean                 - Clean all examples and output directory"
	@echo ""
