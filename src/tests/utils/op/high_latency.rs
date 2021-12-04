use crate::{tests::utils::clock::MonotonicTestClock, traits::IMonotonicClock};

use super::TestOp;

use proptest::{prop_assert, prop_assume, proptest};
use proptest_derive::Arbitrary;

/// `TestOperation` that takes at least some specified duration to run.
#[derive(Arbitrary, Debug)]
pub(crate) struct HighLatencyOp<TClock>
where
    TClock: IMonotonicClock,
{
    clock: TClock,
    latency: TClock::Duration,
}

impl<TClock> HighLatencyOp<TClock>
where
    TClock: IMonotonicClock,
{
    pub fn new(clock: TClock, latency: TClock::Duration) -> Self {
        Self { clock, latency }
    }
}

impl<TClock> TestOp<TClock> for HighLatencyOp<TClock>
where
    TClock: IMonotonicClock,
{
    type Input = ();
    type Output = ();

    fn clock(&mut self) -> &mut TClock {
        &mut self.clock
    }

    fn run(&mut self, _input: Self::Input) -> Self::Output {
        let latency = self.latency.clone();
        self.clock().sleep(&latency);
    }
}

proptest! {
    #[test]
    fn takes_at_least_the_specified_latency_to_return(mut op: HighLatencyOp<MonotonicTestClock>, mut clock: MonotonicTestClock) {
        prop_assume!(clock.now().checked_add(op.latency).is_some(), "Clock should not overflow");

        let (duration, _result) = op.timed_run(());
        prop_assert!(duration >= op.latency, "Actual duration: {:?}", duration);
    }
}
