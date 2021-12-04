mod blocking {
    use crate::MonotonicClock;

    use crate::tests::utils::op::{high_latency::HighLatencyOp, TestOp};

    use std::time::Duration;

    #[test]
    fn high_latency_op_returns_within_specified_timeout_plus_small_overhead() {
        let op_latency = Duration::from_millis(1000);
        let timeout = Duration::from_millis(500);
        let small_overhead = Duration::from_millis(100);

        let (actual_latency, _output) =
            HighLatencyOp::<MonotonicClock>::new(MonotonicClock::default(), op_latency)
                .timed_run(());

        let max_expected_latency = timeout + small_overhead;
        assert!(
            actual_latency <= max_expected_latency,
            "Actual latency of {:?} is not within expected range of {:?}",
            actual_latency,
            max_expected_latency
        );
    }
}
