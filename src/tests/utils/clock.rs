use crate::traits::IMonotonicClock;

use proptest_derive::Arbitrary;

#[derive(Arbitrary, Clone, Debug)]
pub struct MonotonicTestClock {
    now: std::time::Instant,
}

impl From<std::time::Instant> for MonotonicTestClock {
    fn from(now: std::time::Instant) -> Self {
        Self { now }
    }
}

impl Default for MonotonicTestClock {
    fn default() -> Self {
        Self::from(std::time::Instant::now())
    }
}

impl IMonotonicClock for MonotonicTestClock {
    type Instant = std::time::Instant;
    type Duration = std::time::Duration;

    fn now(&self) -> Self::Instant {
        self.now
    }

    fn sleep(&mut self, duration: &Self::Duration) {
        let start_time = self.now();
        match start_time.checked_add(*duration) {
            Some(instant) => {
                self.now = instant;
            }
            None => todo!("deal with overflow case..."),
        }
    }
}
