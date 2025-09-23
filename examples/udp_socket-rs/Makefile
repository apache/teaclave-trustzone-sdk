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

# If _HOST or _TA specific compiler/target are not specified, then use common
# compiler/target for both
CROSS_COMPILE_HOST ?= aarch64-linux-gnu-
CROSS_COMPILE_TA ?= aarch64-linux-gnu-
TARGET_HOST ?= aarch64-unknown-linux-gnu
TARGET_TA ?= aarch64-unknown-linux-gnu
BUILDER ?= cargo
FEATURES ?=

.PHONY: host ta all clean

all: host ta

host:
	$(q)make -C host TARGET=$(TARGET_HOST) \
		CROSS_COMPILE=$(CROSS_COMPILE_HOST)

ta:
	$(q)make -C ta TARGET=$(TARGET_TA) \
		CROSS_COMPILE=$(CROSS_COMPILE_TA) \
		BUILDER=$(BUILDER) \
		FEATURES="$(FEATURES)"

clean:
	$(q)make -C host clean
	$(q)make -C ta clean
