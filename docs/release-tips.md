---
permalink: /trustzone-sdk-docs/release-tips.md
---

# TrustZone SDK Release Tips

For the complete workflow and operational details, please refer to the [Release Guide for the Teaclave Community](https://teaclave.apache.org/release-guide).
This documentation highlights items that are specific to the `TrustZone SDK`.

## Release Schedule & Stages

### Release Schedule

Apache TrustZone SDK follows a quarterly release cycle, aligned with [OP-TEE releases](https://optee.readthedocs.io/en/latest/general/releases.html).

The upcoming release in 2026 is as follows:

| Apache Teaclave SDK Release Version | optee-* Rust crate Release version | OP-TEE Version | OP-TEE Release Date | Teaclave SDK Pre-release on Github (approximately) | Teaclave SDK Finalized Release on Apache and `crates.io` (approximately) |
|-------------------------------------|-------------------------------------|----------------|--------------------|----------------------------------------------------|--------------------------------------------------------------------------|
| v0.8.0  | v0.8.0  | OP-TEE 4.9.0  | 16/Jan/26 | 30/Jan/26 | 13/Feb/26 |
| v0.9.0  | v0.9.0  | OP-TEE 4.10.0 | 17/Apr/26 | 1/May/26  | 15/May/26 |
| v0.10.0 | v0.10.0 | OP-TEE 4.11.0 | 17/Jul/26 | 31/Jul/26 | 14/Aug/26 |
| v0.11.0 | v0.11.0 | OP-TEE 4.12.0 | 16/Oct/26 | 30/Oct/26 | 13/Nov/26 |

**Note:** The table outlines the planned release schedule under normal circumstances. However, if there are no updates to the optee-* crates in the SDK during a given quarter, the release will be skipped and deferred to the following quarter.

According to the Release Documentation in [Teaclave community](https://teaclave.apache.org/release-guide), the approximate timeline for v0.6.0:

- **July 11** – OP-TEE 4.7.0 released
- **July 12–18** – Prepare the release
- **July 19–25** – Publish the pre-release on GitHub and start the vote
- **July 25 – August 8** – Voting period
- **August 8-15** – Post-release steps completed within one week

The timeline is flexible and can be adjusted based on the actual circumstances.


## Specific for TrustZone SDK

### Publish on `crates.io`

We maintain the optee-* Rust crates at <https://crates.io/search?q=optee>, which are released in sync with Apache releases.

If the release manager needs permission to publish these crates, please contact @DemesneGH for access.

After the Apache release is finalized, we need to publish the crates:

```bash
cargo login
cd [each-crates-dir] # should be in correct dependency order, e.g. first optee-teec-sys, then optee-teec
cargo publish --dry-run  # check if ready, will not upload
cargo publish # check and upload
```

### GitHub Action for Drafting Release Notes

We use a GitHub Action to help categorize pull requests and generate a draft of the release notes. This makes the notes more readable and organized. The typical workflow is to first trigger the action, then manually edit the resulting draft as needed.

Manually trigger the Github Action workflow to draft the release notes:

1. Go to Actions → Draft Release Notes
2. Click Run workflow
3. Enter the Version to release (e.g. 0.5.0)
4. Confirm to Run workflow

After the workflow completes, a draft release will appear at:
<https://github.com/apache/teaclave-trustzone-sdk/releases>

The workflow categorizes the PRs according to their labels.

Tips for improving the draft release notes:
- Add a brief summary at the top to highlight the major changes. You can write it manually or generate it using AI.
- Include sections such as “New Contributors” and “Changelog”. Note: Our custom GitHub Action does not generate these sections by default. To get these missing parts, you can click the "Generate Release Notes" button on the release editing page and copy the generated content into the draft.


Please note that once a release is published (including pre-releases), its release notes can no longer be updated via GitHub Actions (manual edits are possible). If you need to update the release notes through Actions (e.g., to revise the release to rc.2 to include the new commits), you must first delete the existing release (e.g. tagged rc.1), then re-trigger the workflow.


### Email template for voting

````
Title: [VOTE] Release Apache Teaclave TrustZone SDK v$VERSION-$RC

Hi all,

I am pleased to be calling this vote for the release of
Apache Teaclave TrustZone SDK $VERSION ($RC).

The release note is available in:
- https://github.com/apache/teaclave-trustzone-sdk/releases/tag/v$VERSION-$RC

The release candidate to be voted over is available at:
- https://dist.apache.org/repos/dist/dev/teaclave/trustzone-sdk-$VERSION-$RC/

The release candidate is signed with a GPG key available at:
- https://downloads.apache.org/teaclave/KEYS 

Instructions to verify the release candidate’s signature:
- https://teaclave.apache.org/download/#verify-the-integrity-of-the-files

The release artifacts have passed all GitHub Actions CI checks. You can also reproduce the build process manually from source using the following commands:

```bash
$ wget https://dist.apache.org/repos/dist/dev/teaclave/trustzone-sdk-$VERSION-$RC/apache-teaclave-trustzone-sdk-$VERSION.tar.gz
$ tar zxvf apache-teaclave-trustzone-sdk-$VERSION.tar.gz
$ cd apache-teaclave-trustzone-sdk-$VERSION
$ docker run -it --rm \
  -v $(pwd):/root/teaclave_sdk_src \
  -w /root/teaclave_sdk_src \
  teaclave/teaclave-trustzone-emulator-nostd-expand-memory:optee-$OPTEE_VERSION
# Inside the docker container:
root@xxxx:~/teaclave_sdk_src# make
```

The vote will be open for at least 72 hours. Anyone can participate
in testing and voting, not just committers, please feel free to try
out the release candidate and provide your votes to this thread
explicitly.

[ ] +1 approve
[ ] +0 no opinion
[ ] -1 disapprove with the reason


Best,
$YOUR_NAME
````


