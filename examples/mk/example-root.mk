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

# Makefile in each example's root directory

# ----------------------------------------------------------------------
# Default configuration
# ----------------------------------------------------------------------
CROSS_COMPILE_HOST ?= aarch64-linux-gnu-
CROSS_COMPILE_TA   ?= aarch64-linux-gnu-
TARGET_HOST        ?= aarch64-unknown-linux-gnu
TARGET_TA          ?= aarch64-unknown-linux-gnu
BUILDER            ?= cargo
FEATURES           ?=

# ----------------------------------------------------------------------
# Default project layout (override as needed)
# ----------------------------------------------------------------------
HOST_DIRS ?= host
TA_DIRS   ?= ta       # default: single TA at ./ta/

.PHONY: all host ta clean emulate

# ----------------------------------------------------------------------
# Default build targets
# ----------------------------------------------------------------------
all: host ta

host:
	$(q)for d in $(HOST_DIRS); do \
		make -C $$d \
			TARGET=$(TARGET_HOST) \
			CROSS_COMPILE=$(CROSS_COMPILE_HOST) || exit $$?; \
	done

ta:
	$(q)for d in $(TA_DIRS); do \
		make -C $$d TA_PATH=$$d \
			TARGET=$(TARGET_TA) \
			CROSS_COMPILE=$(CROSS_COMPILE_TA) \
			BUILDER=$(BUILDER) \
			FEATURES="$(FEATURES)" || exit $$?; \
	done

emulate:
	$(q)for d in $(HOST_DIRS); do \
		make -C $$d emulate TARGET=$(TARGET_HOST) \
			CROSS_COMPILE=$(CROSS_COMPILE_HOST) || exit $$?; \
	done
	$(q)for d in $(TA_DIRS); do \
		make -C $$d emulate TARGET=$(TARGET_TA) \
			CROSS_COMPILE=$(CROSS_COMPILE_TA) \
			BUILDER=$(BUILDER) \
			FEATURES="$(FEATURES)" || exit $$?; \
	done

clean:
	$(q)for d in $(HOST_DIRS); do \
		make -C $$d clean || exit $$?; \
	done
	$(q)for d in $(TA_DIRS); do \
		make -C $$d clean || exit $$?; \
	done
