use proptest::{prop_assert, proptest};
use proptest_derive::Arbitrary;

/// Any operation for which the time it takes to execute is significant.
///
/// A mutable `IMonotonicClock` is injected into all functions that run
/// the operation. This enables testing with either real system time
/// or simulatng the passage of time with a fake clock.
trait TestOperation<TClock>
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

/// `TestOperation` that fails for some number of runs and then succeeds.
#[derive(Arbitrary, Debug)]
struct FallibleOp {
    times_to_fail: u8,
}

impl<T> TestOperation<T> for FallibleOp
where
    T: IMonotonicClock,
{
    type Output = ();
    type Error = ();

    fn run(&mut self, _clock: &mut T) -> Result<Self::Output, Self::Error> {
        if self.times_to_fail == 0 {
            Ok(())
        } else {
            self.times_to_fail -= 1;
            Err(())
        }
    }
}

proptest! {
    #[test]
    fn fallible_op_fails_specified_number_of_times_and_then_succeeds(mut op: FallibleOp, mut clock: MonotonicTestClock) {
        for _ in 0..op.times_to_fail {
            assert!(op.run(&mut clock).is_err());
        }
        assert!(op.run(&mut clock).is_ok());
    }
}

/// `TestOperation` that takes at least some specified duration to run.
#[derive(Arbitrary, Debug)]
struct HighLatencyOp<TClock>
where
    TClock: IMonotonicClock,
{
    latency: TClock::Duration,
}

impl<TClock> TestOperation<TClock> for HighLatencyOp<TClock>
where
    TClock: IMonotonicClock,
{
    type Output = ();
    type Error = ();

    fn run(&mut self, clock: &mut TClock) -> Result<Self::Output, Self::Error> {
        clock.sleep(&self.latency);
        Ok(())
    }
}

proptest! {
    #[test]
    #[ignore = "Adding an arbitrary latency to the current time currently panics on overflow"]
    fn high_latency_op_takes_at_least_the_specified_latency_to_return(mut op: HighLatencyOp<MonotonicTestClock>, mut clock: MonotonicTestClock) {
        let (duration, _result) = op.timed_run(&mut clock);
        prop_assert!(duration >= op.latency);
    }
}

// TODO: add a `BackpressureOp` to support implementing a dynamic retry scheduling policy

/// Intantenous moment in time.
///
/// Conceptually, this is a point on a continuous timeline.
trait IInstant: Clone + Eq + Ord {
    /// Preferred `IDuration` type to use in conjunction with this instant type.
    type Duration: IDuration<Instant = Self>;

    fn checked_duration_since(&self, earlier: Self) -> Option<Self::Duration>;

    fn checked_add(&self, duration: Self::Duration) -> Option<Self>;
}

#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct TestInstant(std::time::Instant);

impl TestInstant {
    fn now() -> Self {
        Self(std::time::Instant::now())
    }
}

impl IInstant for TestInstant {
    type Duration = TestDuration;

    fn checked_duration_since(&self, earlier: Self) -> Option<Self::Duration> {
        self.0.checked_duration_since(earlier.0).map(TestDuration)
    }

    fn checked_add(&self, duration: Self::Duration) -> Option<Self> {
        self.0.checked_add(duration.0).map(TestInstant)
    }
}

/// Amount of time elapsed.
///
/// Conceptually, this is the amount of distance between two points on a timeline.
trait IDuration: Clone + Eq + Ord + Sized {
    /// Preferred `IInstant` type to use in conjunction with this duration type.
    type Instant: IInstant<Duration = Self>;
}

#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct TestDuration(std::time::Duration);

impl IDuration for TestDuration {
    type Instant = TestInstant;
}

/// Monotonically non-decreasing clock.
///
/// The monotonicity contract means that when two samples are taken in succession
/// of the current clock's time the second sample cannot be in the past relative to
/// the first sample.
trait IMonotonicClock {
    type Instant: IInstant<Duration = Self::Duration>;
    type Duration: IDuration<Instant = Self::Instant>;
    type BigUnsigned;

    /// Returns current instant according to this clock.
    fn now(&self) -> Self::Instant;

    /// Sleeps for the specified duration of time.
    ///
    /// Implementations for testing purposes are not expected to actually sleep.
    ///
    /// Returns an infinite precision value indicating how many times the internal time value overflowed.
    fn sleep(&mut self, duration: &Self::Duration) -> Self::BigUnsigned;
}

#[derive(Arbitrary, Debug)]
struct MonotonicTestClock {
    now: TestInstant,
}

impl Default for MonotonicTestClock {
    fn default() -> Self {
        Self {
            now: TestInstant::now(),
        }
    }
}

impl IMonotonicClock for MonotonicTestClock {
    type Instant = TestInstant;
    type Duration = TestDuration;
    type BigUnsigned = num_bigint::BigUint;

    fn now(&self) -> Self::Instant {
        self.now
    }

    fn sleep(&mut self, duration: &Self::Duration) -> Self::BigUnsigned {
        // TODO: deal with unwrap by tracking the number of overflows of internal time value
        let start_time = self.now();
        match start_time.checked_add(*duration) {
            Some(_instant) => 0u8.into(),
            None => todo!("deal with overflow case..."),
        }
    }
}

// Example production `IMonotonicClock`.
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
    type BigUnsigned = num_bigint::BigUint;

    fn now(&self) -> Self::Instant {
        Self::Instant::now()
    }

    /// Implemented using `std::thread::sleep`, which has platform-specific behavior.
    ///
    /// It's unclear if any unexpected behavior could happen if `now + duration` overflows,
    /// but the implementation assumes no such overflow occurs.
    fn sleep(&mut self, duration: &Self::Duration) -> Self::BigUnsigned {
        std::thread::sleep(*duration);
        // TODO: attempt to anticipate overflow scenario so returned
        // value is accurate; may require testing overflow scenario on
        // real systems
        0u8.into()
    }
}
