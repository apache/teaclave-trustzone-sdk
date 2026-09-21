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

TARGET ?= aarch64-unknown-linux-gnu
TARGET_HOST ?= $(TARGET)
TARGET_TA ?= $(TARGET)

.PHONY: host ta all clean

all: host ta

host:
	$(q)make -C host TARGET=$(TARGET_HOST) \
		$(if $(CROSS_COMPILE_HOST),CROSS_COMPILE=$(CROSS_COMPILE_HOST))

ta:
	$(q)make -C ta TARGET=$(TARGET_TA) \
		$(if $(CROSS_COMPILE_TA),CROSS_COMPILE=$(CROSS_COMPILE_TA)) \
		$(if $(TA_SIGN_KEY),TA_SIGN_KEY="$(TA_SIGN_KEY)")

clean:
	$(q)make -C host clean
	$(q)make -C ta clean
