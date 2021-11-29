mod fallible;
mod high_latency;

use crate::traits::{IInstant, IMonotonicClock};

// TODO: add a `BackpressureOp` to support implementing a dynamic retry scheduling policy

/// Any operation for which the time it takes to execute is significant.
///
/// A mutable `IMonotonicClock` is injected into all functions that run
/// the operation. This enables testing with either real system time
/// or simulatng the passage of time with a fake clock.
trait TestOp<TClock>
where
    TClock: IMonotonicClock,
{
    type Output;
    type Error;

    /// Run this operation.
    fn run(&mut self, clock: &mut TClock) -> Result<Self::Output, Self::Error>;

    /// Measure how long it takes to run this operation.
    ///
    /// # Panics
    ///
    /// This function will panic if the duration elapsed while running the operation suggests
    /// the operation took a negative amount of time. Potential reasons this could occur include:
    ///
    /// 1. clock violates its monotically non-decreasing contract, potentially by overflowing
    /// 1. this operation resets the clock to sometime in the past
    fn timed_run(
        &mut self,
        clock: &mut TClock,
    ) -> (TClock::Duration, Result<Self::Output, Self::Error>) {
        let start_time = clock.now();
        let result = self.run(clock);
        let stop_time = clock.now();

        let elapsed = stop_time.checked_duration_since(start_time).expect(
            "clocks are monotonically non-decreasing, so the elapsed time should be non-negative",
        );
        (elapsed, result)
    }
}
