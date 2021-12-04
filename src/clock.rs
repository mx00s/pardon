use crate::traits::IMonotonicClock;

// Example production `IMonotonicClock`.
// TODO: add docs that mention the overflow edge cases that prompted using `prop_assume!`.
#[derive(Debug, Default)]
pub struct MonotonicClock;

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
