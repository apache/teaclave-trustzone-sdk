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

# Default build parameters if not specified.
# TARGET_* is the OP-TEE / `source environment` interface. --arch is derived
# from the triple at the cargo-optee invocation.
TARGET ?= aarch64-unknown-linux-gnu
TARGET_HOST ?= $(TARGET)
TARGET_TA ?= $(TARGET)

# All CA crates and all TA crates now share their respective workspace target
# directories. Do not run multiple per-example Cargo commands concurrently,
# because they may link or clean the same workspace artifacts at the same time.
# Cargo still parallelizes compilation within each individual invocation.
.NOTPARALLEL:

# Define example categories based on std/no-std support
# STD-only examples (require STD=y to build)
STD_ONLY_EXAMPLES = \
	tls_client-rs \
	tls_server-rs \
	secure_db_abstraction-rs

# NO-STD-only examples (require STD to be unset to build)
NO_STD_ONLY_EXAMPLES = \
	mnist-rs

# Common examples (build in both std and no-std modes)
COMMON_EXAMPLES = \
	acipher-rs \
	aes-rs \
	authentication-rs \
	big_int-rs \
	diffie_hellman-rs \
	digest-rs \
	error_handling-rs \
	hello_world-rs \
	hotp-rs \
	inter_ta-rs \
	message_passing_interface-rs \
	property-rs \
	random-rs \
	secure_storage-rs \
	serde-rs \
	supp_plugin-rs \
	tcp_client-rs \
	time-rs \
	udp_socket-rs \
	signature_verification-rs \
	client_pool-rs

# Clean targets
STD_ONLY_EXAMPLES_CLEAN = $(STD_ONLY_EXAMPLES:%=%-clean)
NO_STD_ONLY_EXAMPLES_CLEAN = $(NO_STD_ONLY_EXAMPLES:%=%-clean)
COMMON_EXAMPLES_CLEAN = $(COMMON_EXAMPLES:%=%-clean)

# Build std examples (std-only + common examples)
std-examples: std-only-examples common-examples

# Build no-std examples (no-std-only + common examples)
no-std-examples: no-std-only-examples common-examples

# Build std-only examples
std-only-examples: $(STD_ONLY_EXAMPLES)

# Build no-std-only examples
no-std-only-examples: $(NO_STD_ONLY_EXAMPLES)

# Build common examples (always built)
common-examples: $(COMMON_EXAMPLES)

# Installation uses cargo-optee to locate artifacts for the active std/arch.
BUILD_TARGET ?= all
TA_INSTALL_DIR ?= $(CURDIR)/../out/lib/optee_armtz
CA_INSTALL_DIR ?= $(CURDIR)/../out/usr/bin
PLUGIN_INSTALL_DIR ?= $(CURDIR)/../out/usr/lib/tee-supplicant/plugins
install: BUILD_TARGET = install
install: $(if $(or $(STD),$(filter %-unknown-optee,$(TARGET_TA))),std-examples,no-std-examples)

$(STD_ONLY_EXAMPLES) $(NO_STD_ONLY_EXAMPLES) $(COMMON_EXAMPLES):
	$(q)$(MAKE) -C ca/$@ $(BUILD_TARGET) TARGET=$(TARGET_HOST) \
		INSTALL_DIR="$(abspath $(CA_INSTALL_DIR))" \
		$(if $(CROSS_COMPILE_HOST),CROSS_COMPILE=$(CROSS_COMPILE_HOST))
	$(q)$(MAKE) -C ta/$@ $(BUILD_TARGET) TARGET=$(TARGET_TA) \
		INSTALL_DIR="$(abspath $(TA_INSTALL_DIR))" \
		$(if $(STD),STD="$(STD)") \
		$(if $(CROSS_COMPILE_TA),CROSS_COMPILE=$(CROSS_COMPILE_TA)) \
		$(if $(TA_SIGN_KEY),TA_SIGN_KEY="$(TA_SIGN_KEY)")
	$(if $(filter supp_plugin-rs,$@),$(q)$(MAKE) -C ca/supp_plugin-rs-plugin $(BUILD_TARGET) TARGET=$(TARGET_HOST) \
		INSTALL_DIR="$(abspath $(PLUGIN_INSTALL_DIR))" \
		$(if $(CROSS_COMPILE_HOST),CROSS_COMPILE=$(CROSS_COMPILE_HOST)))

# Clean targets
clean: $(STD_ONLY_EXAMPLES_CLEAN) $(NO_STD_ONLY_EXAMPLES_CLEAN) $(COMMON_EXAMPLES_CLEAN)

$(STD_ONLY_EXAMPLES_CLEAN) $(NO_STD_ONLY_EXAMPLES_CLEAN) $(COMMON_EXAMPLES_CLEAN):
	$(q)make -C ca/$(@:-clean=) clean
	$(q)make -C ta/$(@:-clean=) clean
	$(if $(filter supp_plugin-rs-clean,$@),$(q)make -C ca/supp_plugin-rs-plugin clean)

# Help target
help:
	@echo "Available targets:"
	@echo "  std-examples         - Build std examples (std-only + common)"
	@echo "  no-std-examples      - Build no-std examples (no-std-only + common)"
	@echo "  std-only-examples    - Build std-only examples (requires STD=y)"
	@echo "  no-std-only-examples - Build no-std-only examples (requires STD unset)"
	@echo "  common-examples      - Build examples that work in both modes"
	@echo "  clean                - Clean all examples"

.PHONY: install std-examples no-std-examples std-only-examples no-std-only-examples common-examples \
	$(STD_ONLY_EXAMPLES) $(NO_STD_ONLY_EXAMPLES) $(COMMON_EXAMPLES) \
	$(STD_ONLY_EXAMPLES_CLEAN) $(NO_STD_ONLY_EXAMPLES_CLEAN) $(COMMON_EXAMPLES_CLEAN) clean help
