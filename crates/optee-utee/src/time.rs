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

use crate::{Error, ErrorKind, Result, tee_check};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::time::Duration;
use optee_utee_sys as raw;

/// A time value with millisecond resolution.
///
/// This is a transparent wrapper around `TEE_Time`. The `millis` component is
/// always in `0..1000`; use [`Time::from_parts`] to construct a value and the
/// `*_now` functions to read a clock.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Time(raw::TEE_Time);

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.total_millis() == other.total_millis()
    }
}

impl Eq for Time {}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        self.total_millis().cmp(&other.total_millis())
    }
}

impl Hash for Time {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.total_millis().hash(state);
    }
}

impl Default for Time {
    fn default() -> Self {
        Self(raw::TEE_Time {
            seconds: 0,
            millis: 0,
        })
    }
}

impl Time {
    /// Creates a `Time` from a seconds value and a sub-second millisecond
    /// component.
    ///
    /// # Errors
    ///
    /// Returns `BadParameters` when `millis` is greater than or equal to
    /// `1000`. The component is not normalized silently; an out-of-range value
    /// is a caller bug.
    pub fn from_parts(seconds: u32, millis: u32) -> Result<Self> {
        if millis >= 1_000 {
            return Err(ErrorKind::BadParameters.into());
        }
        Ok(Self(raw::TEE_Time { seconds, millis }))
    }

    /// Returns the whole-seconds component.
    pub const fn seconds(&self) -> u32 {
        self.0.seconds
    }

    /// Returns the sub-second millisecond component, in `0..1000`.
    pub const fn millis(&self) -> u32 {
        self.0.millis
    }

    /// Returns the total number of milliseconds represented by this value.
    pub const fn total_millis(&self) -> u64 {
        self.0.seconds as u64 * 1_000 + self.0.millis as u64
    }

    /// Adds a duration, returning `None` if the result does not fit in a
    /// `TEE_Time`.
    pub fn checked_add(self, duration: Duration) -> Option<Self> {
        Self::from_duration(self.to_duration().checked_add(duration)?)
    }

    /// Subtracts a duration, returning `None` if the result would be negative
    /// or would not fit in a `TEE_Time`.
    pub fn checked_sub(self, duration: Duration) -> Option<Self> {
        Self::from_duration(self.to_duration().checked_sub(duration)?)
    }

    /// Returns the duration from `earlier` to `self`, or `None` when `earlier`
    /// is after `self`.
    pub fn duration_since(self, earlier: Self) -> Option<Duration> {
        self.to_duration().checked_sub(earlier.to_duration())
    }

    /// Reads the current TEE system time.
    ///
    /// The origin is implementation-dependent. The value is monotonic for the
    /// lifetime of a TA instance, so it can be used for deadlines and elapsed
    /// time.
    ///
    /// The underlying C API returns `void`; an implementation error is reported
    /// through `TEE_Panic`, as required by the GP specification.
    pub fn now() -> Self {
        let mut raw_time = raw::TEE_Time {
            seconds: 0,
            millis: 0,
        };
        unsafe {
            raw::TEE_GetSystemTime(&mut raw_time);
        }
        Self(raw_time)
    }

    /// Reads the current REE time.
    ///
    /// This value is as trusted as the REE itself and may be tampered with by
    /// the user or by REE software.
    pub fn ree_now() -> Self {
        let mut raw_time = raw::TEE_Time {
            seconds: 0,
            millis: 0,
        };
        unsafe {
            raw::TEE_GetREETime(&mut raw_time);
        }
        Self(raw_time)
    }

    /// Reads the persistent time of the current Trusted Application.
    ///
    /// The timer is not set automatically; call [`Time::set_ta_time`] first.
    ///
    /// # Errors
    ///
    /// Returns `TimeNotSet`, `TimeNeedsReset`, or `Overflow` from the TEE.
    pub fn ta_now() -> Result<Self> {
        let mut raw_time = raw::TEE_Time {
            seconds: 0,
            millis: 0,
        };
        tee_check(unsafe { raw::TEE_GetTAPersistentTime(&mut raw_time) })?;
        Self::try_from(raw_time)
    }

    /// Sets the persistent time of the current Trusted Application.
    ///
    /// # Errors
    ///
    /// Returns `OutOfMemory` or `StorageNoSpace` from the TEE.
    pub fn set_ta_time(&self) -> Result<()> {
        tee_check(unsafe { raw::TEE_SetTAPersistentTime(&self.0) })
    }

    /// Waits for a timeout on the TEE system timer.
    ///
    /// `None` waits forever. `Some(duration)` waits for that duration; a
    /// duration that cannot be represented as a finite millisecond timeout is
    /// rejected instead of being confused with the infinite sentinel.
    ///
    /// # Errors
    ///
    /// Returns `BadParameters` for an unrepresentable duration and `Cancel`
    /// when the wait is cancelled.
    pub fn wait(timeout: Option<Duration>) -> Result<()> {
        match timeout {
            None => Self::wait_millis(raw::TEE_TIMEOUT_INFINITE),
            Some(duration) => {
                let millis = duration.as_millis();
                if millis >= raw::TEE_TIMEOUT_INFINITE as u128 {
                    return Err(ErrorKind::BadParameters.into());
                }
                Self::wait_millis(millis as u32)
            }
        }
    }

    /// Waits for `timeout` milliseconds on the TEE system timer.
    ///
    /// `raw::TEE_TIMEOUT_INFINITE` (`u32::MAX`) waits forever.
    ///
    /// # Errors
    ///
    /// Returns `Cancel` when the wait is cancelled.
    pub fn wait_millis(timeout: u32) -> Result<()> {
        tee_check(unsafe { raw::TEE_Wait(timeout) })
    }

    fn to_duration(self) -> Duration {
        Duration::from_millis(self.total_millis())
    }

    fn from_duration(duration: Duration) -> Option<Self> {
        let seconds = u32::try_from(duration.as_secs()).ok()?;
        Self::from_parts(seconds, duration.subsec_millis()).ok()
    }
}

impl TryFrom<raw::TEE_Time> for Time {
    type Error = Error;

    fn try_from(raw_time: raw::TEE_Time) -> Result<Self> {
        Self::from_parts(raw_time.seconds, raw_time.millis)
    }
}

impl From<Time> for raw::TEE_Time {
    fn from(time: Time) -> Self {
        time.0
    }
}

impl Time {
    /// Returns the underlying `TEE_Time` without copying.
    pub fn as_raw(&self) -> &raw::TEE_Time {
        &self.0
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s {}ms", self.seconds(), self.millis())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optee_utee_sys::{mock_api, mock_utils::SERIAL_TEST_LOCK};

    /// Guarantees: `from_parts` enforces the millisecond invariant and
    /// `total_millis` composes both components.
    #[test]
    fn from_parts_enforces_invariant_and_totals() {
        let time = Time::from_parts(3, 250).expect("valid time");
        assert_eq!(time.seconds(), 3);
        assert_eq!(time.millis(), 250);
        assert_eq!(time.total_millis(), 3_250);

        for millis in [1_000, 1_001, u32::MAX] {
            let error = Time::from_parts(0, millis).expect_err("millis must be < 1000");
            assert_eq!(error.kind(), ErrorKind::BadParameters);
        }
    }

    /// Guarantees: checked arithmetic carries/borrows milliseconds, rejects
    /// overflow/underflow, and `duration_since` uses the same ordering.
    #[test]
    fn checked_arithmetic_handles_boundaries() {
        let time = Time::from_parts(1, 900).expect("valid time");
        let sum = time.checked_add(Duration::from_millis(250)).expect("fits");
        assert_eq!((sum.seconds(), sum.millis()), (2, 150));

        let max = Time::from_parts(u32::MAX, 999).expect("valid time");
        assert_eq!(max.checked_add(Duration::from_millis(1)), None);

        let time = Time::from_parts(2, 100).expect("valid time");
        let diff = time.checked_sub(Duration::from_millis(250)).expect("fits");
        assert_eq!((diff.seconds(), diff.millis()), (1, 850));

        let time = Time::from_parts(0, 10).expect("valid time");
        assert_eq!(time.checked_sub(Duration::from_millis(11)), None);

        let earlier = Time::from_parts(1, 0).expect("valid time");
        let later = Time::from_parts(2, 500).expect("valid time");
        assert_eq!(
            later.duration_since(earlier),
            Some(Duration::from_millis(1_500))
        );
        assert_eq!(earlier.duration_since(later), None);
    }

    /// Guarantees: each clock constructor reads the matching C API into the
    /// value.
    #[test]
    fn clock_constructors_read_c_values() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let get = mock_api::TEE_GetSystemTime_context();
        get.expect().times(1).returning(|time| unsafe {
            *time = raw::TEE_Time {
                seconds: 11,
                millis: 22,
            }
        });
        let system = Time::now();
        assert_eq!((system.seconds(), system.millis()), (11, 22));

        let get = mock_api::TEE_GetREETime_context();
        get.expect().times(1).returning(|time| unsafe {
            *time = raw::TEE_Time {
                seconds: 33,
                millis: 44,
            }
        });
        let ree = Time::ree_now();
        assert_eq!((ree.seconds(), ree.millis()), (33, 44));

        let get = mock_api::TEE_GetTAPersistentTime_context();
        get.expect().times(1).returning(|time| {
            unsafe {
                *time = raw::TEE_Time {
                    seconds: 55,
                    millis: 66,
                };
            }
            raw::TEE_SUCCESS
        });
        let ta = Time::ta_now().expect("must return the C time");
        assert_eq!((ta.seconds(), ta.millis()), (55, 66));
    }

    /// Guarantees: TA time errors propagate and `set_ta_time` passes the exact
    /// C layout.
    #[test]
    fn ta_time_errors_and_set_layout() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let get = mock_api::TEE_GetTAPersistentTime_context();
        get.expect()
            .times(1)
            .return_const(raw::TEE_ERROR_TIME_NOT_SET);
        let error = Time::ta_now().expect_err("must propagate the error");
        assert_eq!(error.kind(), ErrorKind::TimeNotSet);

        let set = mock_api::TEE_SetTAPersistentTime_context();
        set.expect().times(1).returning(|time| {
            let time = unsafe { *time };
            assert_eq!((time.seconds, time.millis), (77, 88));
            raw::TEE_SUCCESS
        });
        Time::from_parts(77, 88)
            .expect("valid time")
            .set_ta_time()
            .expect("must succeed");
    }

    /// Guarantees: `wait` maps `None` to the infinite sentinel, passes finite
    /// durations through, and rejects the sentinel as a finite duration.
    #[test]
    fn wait_maps_timeouts() {
        let _lock = SERIAL_TEST_LOCK.lock().expect("should get the lock");

        let wait = mock_api::TEE_Wait_context();
        wait.expect().times(1).returning(|timeout| {
            assert_eq!(timeout, raw::TEE_TIMEOUT_INFINITE);
            raw::TEE_SUCCESS
        });
        Time::wait(None).expect("infinite wait must succeed");

        let wait = mock_api::TEE_Wait_context();
        wait.expect().times(1).returning(|timeout| {
            assert_eq!(timeout, 250);
            raw::TEE_SUCCESS
        });
        Time::wait(Some(Duration::from_millis(250))).expect("finite wait must succeed");

        let error = Time::wait(Some(Duration::from_millis(
            raw::TEE_TIMEOUT_INFINITE as u64,
        )))
        .expect_err("the sentinel is not a finite timeout");
        assert_eq!(error.kind(), ErrorKind::BadParameters);
    }
}
