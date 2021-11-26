use proptest::{prop_assert, proptest};
use std::time::{Duration, Instant};

trait TestOperation {
    type Output;
    type Error;

    fn run(&mut self) -> Result<Self::Output, Self::Error>;
}

struct FallibleOp {
    times_to_fail: u8,
}

impl FallibleOp {
    fn new(times_to_fail: u8) -> Self {
        Self { times_to_fail }
    }
}

impl TestOperation for FallibleOp {
    type Output = ();
    type Error = ();

    fn run(&mut self) -> Result<Self::Output, Self::Error> {
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
        for _ in 0..n {
            assert!(op.run().is_err());
        }
        assert!(op.run().is_ok());
    }
}

struct HighLatencyOp {
    latency: std::time::Duration,
}

impl HighLatencyOp {
    fn new(latency: Duration) -> Self {
        Self { latency }
    }
}

impl TestOperation for HighLatencyOp {
    type Output = ();
    type Error = ();

    fn run(&mut self) -> Result<Self::Output, Self::Error> {
        std::thread::sleep(self.latency);
        Ok(())
    }
}

// TODO: implement an injectable clock
proptest! {
    // This test takes too long to execute
    #[test]
    fn high_latency_op_takes_at_least_the_specified_latency_to_return(latency: Duration) {
        let mut op = HighLatencyOp::new(latency);

        let start_time = Instant::now();
        op.run().unwrap();
        let elapsed_time = Instant::now() - start_time;

        prop_assert!(elapsed_time >= latency);
    }
}

// TODO: add a `BackpressureOp` to support implementing a dynamic retry scheduling policy
