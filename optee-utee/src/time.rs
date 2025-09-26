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

use crate::{Error, Result};
use optee_utee_sys as raw;
use core::fmt;

/// A millisecond resolution structure for saving the time.
pub struct Time {
    /// The field for the seconds.
    pub seconds: u32,
    /// The field for the milliseconds within this second.
    pub millis: u32,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    /// Create a new empty time structure.
    pub fn new() -> Self {
        Time {
            seconds: 0,
            millis: 0,
        }
    }

    /// Retrieve the current system time.
    /// The origin of this system time is arbitrary and implementation-dependent.
    /// Different TA instances may even have different system times.
    /// The only guarantee is that the system time is not reset or rolled back during the life of a
    /// given TA instance, so it can be used to compute time differences and operation deadlines.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::Time;
    /// let mut time = Time::new();
    /// time.system_time();
    /// ```
    ///
    /// # Panics
    ///
    /// 1) If the Implementation detects any error.
    pub fn system_time(&mut self) {
        unsafe {
            raw::TEE_GetSystemTime(self as *mut _ as _);
        }
    }

    /// Wait  for the specified number of milliseconds or wait forever if timeout equals
    /// `raw::TEE_TIMEOUT_INFINITE` (0xFFFFFFFF). The waiting timer is `System Time`.
    ///
    /// # Parameters
    ///
    /// 1) `timeout`: The number of milliseconds to wait, or `raw::TEE_TIMEOUT_INFINITE`.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{Time, Result};
    /// # fn main() -> Result<()> {
    /// Time::wait(1000)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `Cancel`: If the wait has been cancelled.
    ///
    /// # Panics
    ///
    /// 1) If the Implementation detects any error.

    pub fn wait(timeout: u32) -> Result<()> {
        match unsafe { raw::TEE_Wait(timeout) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Retrieve the persisten time of the Trusted Application. Since the timer is not
    /// automatically set, this function should be called after [set_ta_time](Time::set_ta_time).
    /// The time is a real-time source of time and the origin of this time is set individually by each Trusted Application.
    /// Also, the time SHALL persist across reboots.
    ///
    /// # Example
    ///
    /// ``` rust,no_run
    /// # use optee_utee::{Time, Result};
    /// # fn main() -> Result<()> {
    /// let mut time = Time::new();
    /// time.system_time();
    /// time.set_ta_time()?;
    /// time.ta_time()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 1) `TimeNotSet`: Time is not set.
    /// 2) `TimeNeedsReset`: Time needs to be reset.
    /// 3) `Overflow`: The number of seconds in the TA Persistent Time overflows the range of a
    ///    `u32`. The field `seconds` is still set to the TA Persistent Time truncated to 32 bits.
    ///
    /// # Panics
    ///
    /// 1) If the Implementation detects any error.
    pub fn ta_time(&mut self) -> Result<()> {
        match unsafe { raw::TEE_GetTAPersistentTime(self as *mut _ as _) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Set the persistent time of the current Trusted Application.
    ///
    /// # Errors
    ///
    /// 1) `OutOfMemory`: If not enough memory is available to complete the operation.
    /// 2) `SotrageNoSpace`: If insufficient storage space is available to complete the operation.
    ///
    /// # Panics
    ///
    /// 1) If the Implementation detects any error.
    pub fn set_ta_time(&self) -> Result<()> {
        match unsafe { raw::TEE_SetTAPersistentTime(self as *const _ as _) } {
            raw::TEE_SUCCESS => Ok(()),
            code => Err(Error::from_raw_error(code)),
        }
    }

    /// Retrieve the current REE system time.
    /// The time is as trusted as the REE itself and may also be tampered by the user.
    ///
    /// Panics
    ///
    /// 1) If the Implementation detects any error.
    pub fn ree_time(&mut self) {
        unsafe {
            raw::TEE_GetREETime(self as *mut _ as _);
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(second: {}, millisecond: {})",
            self.seconds, self.millis
        )
    }
}
