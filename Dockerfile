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

FROM debian:trixie
MAINTAINER Teaclave Contributors <dev@teaclave.apache.org>
LABEL org.opencontainers.image.description "Github CI Runner"
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies for building OP-TEE
COPY setup_optee_dependencies.sh /tmp/setup_optee_dependencies.sh
RUN /tmp/setup_optee_dependencies.sh && rm -f /tmp/setup_optee_dependencies.sh

# OP-TEE stopped building QEMU from source for the QEMU targets in
# https://github.com/OP-TEE/build/commit/4e38530, so `make check` now uses the
# distribution qemu-system-aarch64. Debian trixie ships QEMU 10, the version
# upstream CI validates against. Only the OP-TEE test workflow needs QEMU, so
# it is installed here instead of in setup_optee_dependencies.sh.
RUN apt-get update && \
    apt-get install -y --no-install-recommends qemu-system-arm && \
    rm -rf /var/lib/apt/lists/*
