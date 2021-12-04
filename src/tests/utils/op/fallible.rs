use crate::{tests::utils::clock::MonotonicTestClock, traits::IMonotonicClock};

use super::TestOp;

use proptest::{prop_assert, proptest};
use proptest_derive::Arbitrary;

/// `TestOperation` that fails for some number of runs and then succeeds.
#[derive(Arbitrary, Debug)]
struct FallibleOp<TClock>
where
    TClock: IMonotonicClock,
{
    clock: TClock,
    times_to_fail: u8,
}

impl<TClock> TestOp<TClock> for FallibleOp<TClock>
where
    TClock: IMonotonicClock,
{
    type Input = ();
    type Output = Result<(), ()>;

    fn clock(&mut self) -> &mut TClock {
        &mut self.clock
    }

    fn run(&mut self, _input: Self::Input) -> Self::Output {
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
    fn fails_specified_number_of_times_and_then_succeeds(mut op: FallibleOp<MonotonicTestClock>) {
        for _ in 0..op.times_to_fail {
            prop_assert!(op.run(()).is_err());
        }
        prop_assert!(op.run(()).is_ok());
    }
}
