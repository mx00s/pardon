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

    // This implementation works for the unit tests over the real monotonic clock; however,
    // it doesn't work well with the test clock yet because there's no inherent bias toward
    // whichever thread is *expected* to finish first with a normal clock.
    fn timed_run_with_timeout(
        self,
        input: Self::Input,
        timeout: TClock::Duration,
    ) -> (TClock::Duration, Result<Self::Output, ()>)
    where
        Self: Send + Sized + Clone + 'static,
        <Self as TestOp<TClock>>::Input: Send + 'static,
        <Self as TestOp<TClock>>::Output: Send + 'static,
        <TClock as IMonotonicClock>::Duration: Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut op_for_timeout = self.clone();
        let tx_for_timeout = tx.clone();
        std::thread::spawn(move || {
            op_for_timeout.clock().sleep(&timeout);
            tx_for_timeout.send((timeout, Err(()))).unwrap();
        });

        let mut op_for_run = self;
        let tx_for_run = tx;
        std::thread::spawn(move || {
            std::thread::yield_now();
            let (latency, output) = op_for_run.timed_run(input);
            tx_for_run.send((latency, Ok(output))).unwrap()
        });

        rx.recv().unwrap()
    }
}
