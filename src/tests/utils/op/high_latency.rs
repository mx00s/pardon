use crate::{tests::utils::clock::MonotonicTestClock, traits::IMonotonicClock};

use super::TestOp;

use proptest::{prop_assert, prop_assume, proptest};
use proptest_derive::Arbitrary;

/// `TestOperation` that takes at least some specified duration to run.
#[derive(Arbitrary, Debug)]
struct HighLatencyOp<TClock>
where
    TClock: IMonotonicClock,
{
    clock: TClock,
    latency: TClock::Duration,
}

impl<TClock> TestOp<TClock> for HighLatencyOp<TClock>
where
    TClock: IMonotonicClock,
{
    type Output = ();
    type Error = ();

    fn clock(&mut self) -> &mut TClock {
        &mut self.clock
    }

    fn run(&mut self) -> Result<Self::Output, Self::Error> {
        let latency = self.latency.clone();
        self.clock().sleep(&latency);
        Ok(())
    }
}

proptest! {
    #[test]
    fn takes_at_least_the_specified_latency_to_return(mut op: HighLatencyOp<MonotonicTestClock>, mut clock: MonotonicTestClock) {
        prop_assume!(clock.now().checked_add(op.latency).is_some(), "Clock should not overflow");

        let (duration, _result) = op.timed_run();
        prop_assert!(duration >= op.latency, "Actual duration: {:?}", duration);
    }
}
