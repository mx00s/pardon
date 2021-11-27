use std::ops::{Add, Sub};

use proptest::proptest;
use proptest_derive::Arbitrary;

trait TestOperation<TClock>
where
    TClock: IClock,
{
    type Output;
    type Error;

    fn run(&mut self, clock: &mut TClock) -> Result<Self::Output, Self::Error>;
}

struct FallibleOp {
    times_to_fail: u8,
}

impl FallibleOp {
    fn new(times_to_fail: u8) -> Self {
        Self { times_to_fail }
    }
}

impl<T> TestOperation<T> for FallibleOp
where
    T: IClock,
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
    fn fallible_op_fails_expected_number_of_times_and_then_succeeds(mut clock: TestClock, fail_count: u8) {
        let mut op = FallibleOp::new(fail_count);
        for _ in 0..fail_count {
            assert!(op.run(&mut clock).is_err());
        }
        assert!(op.run(&mut clock).is_ok());
    }
}

struct HighLatencyOp<TClock>
where
    TClock: IClock,
{
    latency: TClock::Duration,
}

impl<TClock> HighLatencyOp<TClock>
where
    TClock: IClock,
{
    fn new(latency: TClock::Duration) -> Self {
        Self { latency }
    }

    fn verify_op_takes_at_least_specified_latency_to_return(
        mut clock: TClock,
        latency: TClock::Duration,
    ) {
        let mut op = Self::new(latency.clone());

        // TODO: make a stopwatch abstraction
        let start_time = clock.now().clone();
        op.run(&mut clock).unwrap();
        let stop_time = clock.now().clone();

        let elapsed = (stop_time - start_time).expect("TestClock is monotonic");
        assert!(elapsed >= latency);
    }
}

impl<TClock> TestOperation<TClock> for HighLatencyOp<TClock>
where
    TClock: IClock,
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
    fn high_latency_op_takes_at_least_the_specified_latency_to_return(clock: TestClock, latency: TestDuration) {
        HighLatencyOp::verify_op_takes_at_least_specified_latency_to_return(clock, latency);
    }
}

// TODO: add a `BackpressureOp` to support implementing a dynamic retry scheduling policy

// TODO: implement an injectable clock
trait IInstant:
    Clone
    + Eq
    + Ord
    + Add<Self::Duration, Output = Option<Self>>
    + Sub<Self, Output = Option<Self::Duration>>
{
    type Duration: IDuration<Instant = Self>;
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
}

trait IDuration: Clone + Eq + Ord + Sized {
    type Instant: IInstant<Duration = Self>;
}

#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct TestDuration(std::time::Duration);

impl IDuration for TestDuration {
    type Instant = TestInstant;
}

impl Add<TestDuration> for TestInstant {
    type Output = Option<Self>;

    fn add(self, other: TestDuration) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
}

impl Sub<TestInstant> for TestInstant {
    type Output = Option<TestDuration>;

    fn sub(self, other: Self) -> Self::Output {
        self.0.checked_duration_since(other.0).map(TestDuration)
    }
}

trait IClock {
    type Instant: IInstant<Duration = Self::Duration>;
    type Duration: IDuration<Instant = Self::Instant>;

    fn now(&self) -> &Self::Instant;

    fn sleep(&mut self, duration: &Self::Duration);
}

// TODO: implement two test clocks
//   1. use instant and duration types that are susceptible to overflow, like std::time::{Instant, Duration}
//   2. truly monotonic clock in which instant and duration types are no susceptible to overflow

#[derive(Arbitrary, Debug)]
struct TestClock {
    now: TestInstant,
}

impl Default for TestClock {
    fn default() -> Self {
        Self {
            now: TestInstant::now(),
        }
    }
}

impl IClock for TestClock {
    type Instant = TestInstant;
    type Duration = TestDuration;

    fn now(&self) -> &Self::Instant {
        &self.now
    }

    fn sleep(&mut self, duration: &Self::Duration) {
        // TODO: deal with unwrap, e.g. by overflowing or saturating
        self.now = (self.now + duration.clone()).unwrap();
    }
}
