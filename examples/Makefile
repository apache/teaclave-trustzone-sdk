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

# Default build parameters if not specified
TARGET ?= aarch64-unknown-linux-gnu
CROSS_COMPILE ?= aarch64-linux-gnu-
CROSS_COMPILE_HOST ?= $(CROSS_COMPILE)
CROSS_COMPILE_TA ?= $(CROSS_COMPILE)
TARGET_HOST ?= $(TARGET)
TARGET_TA ?= $(TARGET)
BUILDER ?= cargo
FEATURES ?=

# Define example categories based on std/no-std support
# STD-only examples (require STD=y to build)
STD_ONLY_EXAMPLES = \
	message_passing_interface-rs \
	serde-rs \
	tls_client-rs \
	tls_server-rs \
	secure_db_abstraction-rs

# NO-STD-only examples (require STD to be unset to build)
NO_STD_ONLY_EXAMPLES = \
	mnist-rs \
	build_with_optee_utee_sys-rs

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
	property-rs \
	random-rs \
	secure_storage-rs \
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

# Individual example build rules
$(STD_ONLY_EXAMPLES) $(NO_STD_ONLY_EXAMPLES) $(COMMON_EXAMPLES):
	$(q)make -C $@ TARGET_HOST=$(TARGET_HOST) \
		TARGET_TA=$(TARGET_TA) \
		CROSS_COMPILE_HOST=$(CROSS_COMPILE_HOST) \
		CROSS_COMPILE_TA=$(CROSS_COMPILE_TA) \
		TA_DEV_KIT_DIR=$(TA_DEV_KIT_DIR) \
		OPTEE_CLIENT_EXPORT=$(OPTEE_CLIENT_EXPORT) \
		BUILDER=$(BUILDER) \
		FEATURES="$(FEATURES)"

# Clean targets
clean: $(STD_ONLY_EXAMPLES_CLEAN) $(NO_STD_ONLY_EXAMPLES_CLEAN) $(COMMON_EXAMPLES_CLEAN)

$(STD_ONLY_EXAMPLES_CLEAN) $(NO_STD_ONLY_EXAMPLES_CLEAN) $(COMMON_EXAMPLES_CLEAN):
	$(q)make -C $(@:-clean=) clean

# Help target
help:
	@echo "Available targets:"
	@echo "  std-examples         - Build std examples (std-only + common)"
	@echo "  no-std-examples      - Build no-std examples (no-std-only + common)"
	@echo "  std-only-examples    - Build std-only examples (requires STD=y)"
	@echo "  no-std-only-examples - Build no-std-only examples (requires STD unset)"
	@echo "  common-examples      - Build examples that work in both modes"
	@echo "  clean                - Clean all examples"

.PHONY: std-examples no-std-examples std-only-examples no-std-only-examples common-examples \
	$(STD_ONLY_EXAMPLES) $(NO_STD_ONLY_EXAMPLES) $(COMMON_EXAMPLES) \
	$(STD_ONLY_EXAMPLES_CLEAN) $(NO_STD_ONLY_EXAMPLES_CLEAN) $(COMMON_EXAMPLES_CLEAN) clean help
