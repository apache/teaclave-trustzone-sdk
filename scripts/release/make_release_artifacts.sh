#!/bin/bash

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

set -euo pipefail

# ---------------- Modify for Each Release ---------------
RELEASE_VERSION="0.5.0"
RC_NUMBER="1"
GPG_KEY_UID="YOUR_KEY_UID"

ASF_USERNAME="${ASF_USERNAME:-your_asf_username}"
ASF_PASSWORD="${ASF_PASSWORD:-your_asf_password}"
# --------------------------------------------------------

# ------------------- Specific for Repo ------------------
# Name of Github Repo
REPO_NAME="teaclave-trustzone-sdk"
# Name of Apache release artifacts
TAR_NAME="apache-teaclave-trustzone-sdk-${RELEASE_VERSION}"

# SVN directory to put these artifacts
SVN_RC_DIR="trustzone-sdk-${RELEASE_VERSION}-rc.${RC_NUMBER}"
SVN_FINAL_DIR="trustzone-sdk-${RELEASE_VERSION}"
# --------------------------------------------------------

WORK_BASE_DIR="teaclave-release-tmp"
TAR_TOP_DIR_NAME="apache-${REPO_NAME}-${RELEASE_VERSION}"
TAG="v${RELEASE_VERSION}-rc.${RC_NUMBER}"

# SVN repository URLs
SVN_DEV_BASE="https://dist.apache.org/repos/dist/dev/teaclave"
SVN_RELEASE_BASE="https://dist.apache.org/repos/dist/release/teaclave"

show_usage() {
    echo "Usage: $0 <command>"
    echo
    echo "  prepare       : Package, sign, and verify release artifacts."
    echo "  upload        : Verify existing artifacts and upload to Apache dist/dev SVN."
    echo "  finalize      : Promote RC to final release and clean up RC artifacts."
    echo "  clean         : Remove temporary working directory and artifacts."
    echo
    echo "Set these variables in the script before run:"
    echo "  RELEASE_VERSION"
    echo "  RC_NUMBER"
    echo "  GPG_KEY_UID"
    echo "  ASF_USERNAME (can override via env)"
    echo "  ASF_PASSWORD (can override via env)"
    echo
    exit 1
}

verify_artifacts() {
    echo "[INFO] Verifying artifacts: ${TAR_NAME}.tar.gz"

    wget -q -O KEYS "${SVN_RELEASE_BASE}/KEYS"

    mkdir -p tmp-keyring

    if ! gpg --no-default-keyring --homedir tmp-keyring --import KEYS; then
        echo "[WARN] gpg import returned an error. This may be caused by gpg-agent, but keys were still imported. Please make sure the verification passed in next step."
    fi

    echo "[INFO] Verifying GPG signature..."
    gpgv --keyring tmp-keyring/pubring.kbx "${TAR_NAME}.tar.gz.asc" "${TAR_NAME}.tar.gz"

    echo "[INFO] Verifying SHA512 checksum..."
    sha512sum -c "${TAR_NAME}.tar.gz.sha512"

    echo "[SUCCESS] Artifact verification passed."
    rm -r tmp-keyring
}

# ---------------- Main ----------------

if [ $# -eq 0 ]; then
    show_usage
fi

case "$1" in
    prepare)
        echo "[INFO] Preparing release artifacts..."

        mkdir -p "$WORK_BASE_DIR"
        cd "$WORK_BASE_DIR"

        echo "[INFO] Downloading tarball from GitHub tag: $TAG"
        wget "https://github.com/apache/${REPO_NAME}/archive/refs/tags/${TAG}.tar.gz"
        tar xzvf "${TAG}.tar.gz"
        
        mv "${REPO_NAME}-${RELEASE_VERSION}-rc.${RC_NUMBER}" "${TAR_TOP_DIR_NAME}"

        echo "[INFO] Normalizing tarball metadata..."
        MTIME=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M:%S" "${TAR_TOP_DIR_NAME}/LICENSE")

        # On macOS, use gnu-tar
        if [[ $(uname) == "Darwin" ]]; then
            if ! command -v gtar &> /dev/null; then
                echo "[ERROR] GNU tar (gtar) is required. Please run: brew install gnu-tar"
                exit 1
            fi
            TAR_CMD=gtar
        else
            TAR_CMD=tar
        fi

        ${TAR_CMD} --sort=name \
            --owner=0 --group=0 --numeric-owner \
            --mtime="${MTIME}" \
            -cvf - "${TAR_TOP_DIR_NAME}" | gzip -n > "${TAR_NAME}.tar.gz"

        echo "[INFO] Signing..."
        gpg --detach-sign --armor -u "${GPG_KEY_UID}" "${TAR_NAME}.tar.gz"
        sha512sum "${TAR_NAME}.tar.gz" > "${TAR_NAME}.tar.gz.sha512"

        verify_artifacts
        ;;

    upload)
        echo "[INFO] Uploading verified artifacts to Apache SVN..."

        cd "$WORK_BASE_DIR"
        verify_artifacts

        echo "[INFO] Uploading to SVN..."
        svn co --depth=files "${SVN_DEV_BASE}" svn-dev-teaclave
        cd svn-dev-teaclave

        mkdir "${SVN_RC_DIR}"
        cp ../${TAR_NAME}.tar.gz{,.asc,.sha512} "${SVN_RC_DIR}/"
        svn add "${SVN_RC_DIR}"
        svn ci --username "${ASF_USERNAME}" --password "${ASF_PASSWORD}" -m "Add ${SVN_RC_DIR}"

        echo "[SUCCESS] Uploaded ${SVN_RC_DIR} to Apache dist/dev SVN."
        ;;


    finalize)
        echo "[INFO] Finalizing release: promoting RC to final..."

        mkdir -p "$WORK_BASE_DIR"
        cd "$WORK_BASE_DIR"
 
        # If the directory already exists, this script will not remove or overwrite it, let the users handle this case.
        if [ -d svn-dev-teaclave ]; then
            echo "[ERROR] Directory 'svn-dev-teaclave' already exists. Please remove it before proceeding."
            exit 1
        fi

        echo "[INFO] Checking out svn-dev-teaclave directory..."
        svn co "${SVN_DEV_BASE}" svn-dev-teaclave
        cd svn-dev-teaclave

        echo "[INFO] Creating final release folder: ${SVN_FINAL_DIR}"
        mkdir "${SVN_FINAL_DIR}"
        cp -r "${SVN_RC_DIR}/"* "${SVN_FINAL_DIR}/"
        svn add "${SVN_FINAL_DIR}"
        svn ci --username "${ASF_USERNAME}" --password "${ASF_PASSWORD}" -m "Add ${REPO_NAME} ${RELEASE_VERSION} final release"

        echo "[INFO] Copying from dev to release repository..."
        export SVN_EDITOR=true
        svn cp \
            "${SVN_DEV_BASE}/${SVN_FINAL_DIR}/" \
            "${SVN_RELEASE_BASE}/${SVN_FINAL_DIR}/" \
            -m "Promote ${REPO_NAME} ${RELEASE_VERSION} to release"

        echo "[INFO] Removing RC folder: ${SVN_RC_DIR}"
        svn delete "${SVN_RC_DIR}"
        svn ci --username "${ASF_USERNAME}" --password "${ASF_PASSWORD}" \
            -m "${REPO_NAME}: delete old RCs"

        echo "[SUCCESS] Finalized release ${RELEASE_VERSION}"
        ;;

    clean)
        echo "[INFO] Cleaning up working directory: ${WORK_BASE_DIR}"
        rm -rf "${WORK_BASE_DIR}"
        echo "[SUCCESS] Cleaned up."
        ;;

    *)
        show_usage
        ;;
esac

