# Operation Guide for Integration with Each OP-TEE Release

OP-TEE follows a quarterly release cycle, and it is essential to keep the TrustZone SDK 
up to date with the latest OP-TEE versions.

This guide outlines the steps required to integrate with each new OP-TEE release, 
including building QEMU images, uploading artifacts to nightlies, and updating the 
development environment. 

## Steps

### 1. CI Build QEMU Image

The CI will build with **the latest OP-TEE tag**. The output artifacts are available 
on the GitHub Actions page.

**To trigger the build:**
1. Go to Actions → Build Test Qemu Image → Run workflow
2. Select branch: main

**Important:** While we build against the upstream OP-TEE repository (not our own 
codebase), our repository contains `.patch/` files that may be applied during the build. 
Therefore, ensure you select the `main` branch to include any necessary patches.

Once the CI workflow completes, the generated artifacts will be accessible on the 
Actions page.

**Example:** https://github.com/apache/teaclave-trustzone-sdk/actions/runs/18874982493

### 2. Download Artifacts and Upload to Nightlies

This step requires an Apache account. You need to download the CI artifacts and upload 
them to the nightlies repository.

#### Nightlies Repository Overview

**Location:** https://nightlies.apache.org/teaclave/teaclave-trustzone-sdk/

For each OP-TEE release, we maintain a comprehensive set of images across multiple 
architectures (aarch64, x86_64). These images serve dual purposes:
1. **CI Infrastructure:** Enable automated testing of TAs on QEMUv8 platforms
2. **Developer Environment:** Provide ready-to-use emulation environments for quick 
   TA development and testing

#### File Structure

For each OP-TEE release (e.g., 4.8.0), the following files are generated:
- `aarch64-optee-4.8.0-qemuv8-ubuntu-24.04-expand-ta-memory.tar.gz`
- `aarch64-optee-4.8.0-qemuv8-ubuntu-24.04.tar.gz`
- `x86_64-optee-4.8.0-qemuv8-ubuntu-24.04-expand-ta-memory.tar.gz`
- `x86_64-optee-4.8.0-qemuv8-ubuntu-24.04.tar.gz`

#### Image Types

- **Standard images**: By default, our examples use standard images, which are built 
  on the OP-TEE repo without changes.
- **Expand-ta-memory images**: Used by TAs that need large memory, such as TLS examples. 
  The patch is applied based on the OP-TEE repo codebase. See 
  [Expanding TA Secure Memory on QEMUv8](expanding-ta-secure-memory-on-qemuv8.md) 
  for details.

#### Upload Process

1. **Download from Action artifacts:**

   Download all four artifacts from the CI Actions page.
   Note: If downloading from GitHub Actions, the file will be in zip format. You should 
   run `unzip` to extract the `*.tar.gz` files.

2. **Upload to nightlies:**
   
   You need an Apache account. See [Becoming a Member](https://teaclave.apache.org/becoming-a-member) 
   for more information.

   Upload each of the four artifacts using curl. Example command:
   ```bash
   curl -u YOUR_ASF_ID \
        -T ./aarch64-optee-4.8.0-qemuv8-ubuntu-24.04.tar.gz \
        "https://nightlies.apache.org/teaclave/teaclave-trustzone-sdk/"
   ```

### 3. PR to Bump optee-version.txt

Update the OP-TEE version to enable the environment to download the correct OP-TEE 
image version.

1. Update the version in: 
   https://github.com/apache/teaclave-trustzone-sdk/blob/main/optee-version.txt
2. Create a PR with the version bump
3. Merge the PR after CI passes

### 4. Build and Publish Dev Docker

After step 3 PR is merged, rebuild the development Docker image.

1. Use the build script: 
   https://github.com/apache/teaclave-trustzone-sdk/blob/main/scripts/release/build_dev_docker.sh
2. Make sure it is tagged as "latest" and publish the new image to Docker Hub: 
   https://hub.docker.com/u/teaclave
   
   This operation requires access to the Teaclave Docker Hub organization. Please 
   contact project maintainers for assistance.

### 5. Update CI Configuration for Development Docker

Following the Docker image publication, update the CI workflow to reference the new 
OP-TEE version.

**Configuration Location:** 
https://github.com/apache/teaclave-trustzone-sdk/blob/main/.github/workflows/ci.yml#L72

**Affected Jobs:**
- `test-nostd-build-on-dev-docker`
- `test-std-build-on-dev-docker`