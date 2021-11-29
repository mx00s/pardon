use crate::{tests::utils::clock::MonotonicTestClock, traits::IMonotonicClock};

use super::TestOp;

use proptest::{prop_assert, proptest};
use proptest_derive::Arbitrary;

/// `TestOperation` that fails for some number of runs and then succeeds.
#[derive(Arbitrary, Debug)]
struct FallibleOp {
    times_to_fail: u8,
}

impl<T> TestOp<T> for FallibleOp
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
    fn fails_specified_number_of_times_and_then_succeeds(mut op: FallibleOp, mut clock: MonotonicTestClock) {
        for _ in 0..op.times_to_fail {
            prop_assert!(op.run(&mut clock).is_err());
        }
        prop_assert!(op.run(&mut clock).is_ok());
    }
}
