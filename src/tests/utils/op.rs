pub(crate) mod fallible;
pub(crate) mod latency;

use crate::traits::{IInstant, IMonotonicClock};

// TODO: add a `BackpressureOp` to support implementing a dynamic retry scheduling policy

/// Any operation for which the time it takes to execute is significant.
///
/// A mutable `IMonotonicClock` is injected into all functions that run
/// the operation. This enables testing with either real system time
/// or simulatng the passage of time with a fake clock.
pub(crate) trait TestOp<TClock>
where
    TClock: IMonotonicClock,
{
    type Input;
    type Output;

    fn clock(&mut self) -> &mut TClock;

    /// Run this operation.
    fn run(&mut self, input: Self::Input) -> Self::Output;

    /// Measure how long it takes to run this operation.
    ///
    /// # Panics
    ///
    /// This function will panic if the duration elapsed while running the operation suggests
    /// the operation took a negative amount of time. Potential reasons this could occur include:
    ///
    /// 1. clock violates its monotically non-decreasing contract, potentially by overflowing
    /// 1. this operation resets the clock to sometime in the past
    fn timed_run(&mut self, input: Self::Input) -> (TClock::Duration, Self::Output) {
        let start_time = self.clock().now();
        let result = self.run(input);
        let stop_time = self.clock().now();

        let elapsed = stop_time.checked_duration_since(start_time).expect(
            "clocks are monotonically non-decreasing, so the elapsed time should be non-negative",
        );
        (elapsed, result)
    }

    fn timed_run_with_timeout(
        &mut self,
        _input: Self::Input,
        timeout: TClock::Duration,
    ) -> (TClock::Duration, Result<Self::Output, ()>) {
        // TODO: call `timed_run` in a way that can be interrupted

        self.clock().sleep(&timeout);
        (timeout, Err(()))
    }
}
