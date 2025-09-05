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

DIST_DIRS := tee-cli tls-host tls-ta
SUBDIRS := $(DIST_DIRS)

.PHONY: all dist clean clippy $(SUBDIRS)

all: dist

test: $(TEST_DIRS)

dist: $(DIST_DIRS)

$(SUBDIRS):
	@$(MAKE) -s -C $@

clippy:
	@for dir in $(SUBDIRS); do \
		$(MAKE) -s -C $$dir clippy; \
	done

clean:
	@for dir in $(SUBDIRS); do \
		$(MAKE) -s -C $$dir clean; \
	done
