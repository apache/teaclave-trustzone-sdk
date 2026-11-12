// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Check run before generating bindings: the TA dev kit version.

use std::error::Error;
use std::fs;
use std::path::Path;

/// Fails unless the TA dev kit was exported from the OP-TEE release pinned in
/// `optee-version.txt`.
///
/// The version is read from `host_include/conf.mk`, which every exported dev kit
/// carries (`# TEE version X.Y.Z`), so no OP-TEE source checkout is needed. A
/// dev kit that does not expose the version is only a warning: there is nothing
/// to compare against, but the headers may still be usable.
pub(super) fn check_optee_version(
    expected: &str,
    ta_dev_kit_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let path = ta_dev_kit_dir.join("host_include/conf.mk");
    let Ok(conf) = fs::read_to_string(&path) else {
        eprintln!(
            "warning: cannot read {}; skipping the version check against optee-version.txt \
             ({expected})",
            path.display()
        );
        return Ok(());
    };

    const VERSION_PREFIX: &str = "# TEE version ";
    let Some(found) = conf
        .lines()
        .find_map(|line| line.strip_prefix(VERSION_PREFIX))
    else {
        eprintln!(
            "warning: {} has no `# TEE version` line; skipping the version check against \
             optee-version.txt ({expected})",
            path.display()
        );
        return Ok(());
    };

    let found = found.trim();
    if found != expected {
        return Err(format!(
            "the TA dev kit is from OP-TEE {found}, but optee-version.txt requests {expected}; \
             export the dev kit from the {expected} tag or update optee-version.txt"
        )
        .into());
    }

    Ok(())
}
