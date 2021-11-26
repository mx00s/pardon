use std::ops::{Add, Sub};

use proptest::{prop_assert, proptest};
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
    fn fallible_op_fails_expected_number_of_times_and_then_succeeds(n: u8) {
        let mut op = FallibleOp::new(n);
        let mut clock = TestClock::default();
        for _ in 0..n {
            assert!(op.run(&mut clock).is_err());
        }
        assert!(op.run(&mut clock).is_ok());
    }
}

struct HighLatencyOp<T>
where
    T: IClock,
{
    latency: T::Duration,
}

impl<T> HighLatencyOp<T>
where
    T: IClock,
{
    fn new(latency: T::Duration) -> Self {
        Self { latency }
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
    fn high_latency_op_takes_at_least_the_specified_latency_to_return(latency: TestDuration) {
        let mut op = HighLatencyOp::new(latency);
        let mut clock = TestClock::default();

        let start_time = clock.now().clone();
        op.run(&mut clock).unwrap();
        let elapsed: TestDuration = (*clock.now() - start_time).expect("TestClock is monotonic");

        prop_assert!(elapsed >= latency);
    }
}

// TODO: add a `BackpressureOp` to support implementing a dynamic retry scheduling policy

// TODO: implement an injectable clock
trait IInstant: Clone + Eq + Ord {}

#[derive(Arbitrary, Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct TestInstant(std::time::Instant);

impl TestInstant {
    fn now() -> Self {
        Self(std::time::Instant::now())
    }
}

impl IInstant for TestInstant {}

trait IDuration: Clone + Eq + Ord + Sized {
    type Instant: IInstant
        + Add<Self, Output = Option<Self::Instant>>
        + Sub<Self::Instant, Output = Option<Self>>;
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
    type Instant: IInstant;
    type Duration: IDuration<Instant = Self::Instant>;

    fn now(&self) -> &Self::Instant;

    fn sleep(&mut self, duration: &Self::Duration);
}

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
