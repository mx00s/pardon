mod test_op;

use crate::traits::{IDuration, IInstant, IMonotonicClock};

use proptest_derive::Arbitrary;

// TODO: add a `BackpressureOp` to support implementing a dynamic retry scheduling policy

#[derive(Arbitrary, Debug)]
struct MonotonicTestClock {
    now: std::time::Instant,
}

impl Default for MonotonicTestClock {
    fn default() -> Self {
        Self {
            now: std::time::Instant::now(),
        }
    }
}

impl IMonotonicClock for MonotonicTestClock {
    type Instant = std::time::Instant;
    type Duration = std::time::Duration;

    fn now(&self) -> Self::Instant {
        self.now
    }

    fn sleep(&mut self, duration: &Self::Duration) {
        let start_time = self.now();
        match start_time.checked_add(*duration) {
            Some(instant) => {
                self.now = instant;
            }
            None => todo!("deal with overflow case..."),
        }
    }
}

// Example production `IMonotonicClock`.
// TODO: add docs that mention the overflow edge cases that prompted using `prop_assume!`.
#[derive(Debug)]
struct MonotonicClock;

impl IInstant for std::time::Instant {
    type Duration = std::time::Duration;

    fn checked_duration_since(&self, earlier: Self) -> Option<Self::Duration> {
        self.checked_duration_since(earlier)
    }

    fn checked_add(&self, duration: Self::Duration) -> Option<Self> {
        self.checked_add(duration)
    }
}

impl IDuration for std::time::Duration {
    type Instant = std::time::Instant;
}

impl IMonotonicClock for MonotonicClock {
    type Instant = std::time::Instant;
    type Duration = std::time::Duration;

    fn now(&self) -> Self::Instant {
        Self::Instant::now()
    }

    /// Implemented using `std::thread::sleep`, which has platform-specific behavior.
    ///
    /// It's unclear if any unexpected behavior could happen if `now() + duration` overflows,
    /// but the implementation makes a best effort to catch and report panics for theoretical
    /// edge cases like this.
    #[tracing::instrument]
    fn sleep(&mut self, duration: &Self::Duration) {
        // In principle `std::thread::sleep` *might* panic, depending on the platform-specific implementation,
        // e.g. perhaps if the current time plus the duration would overflow the clock.
        let sleep = || std::thread::sleep(*duration);
        if let Err(panic) = std::panic::catch_unwind(sleep) {
            tracing::error!(?panic);
        }
    }
}
