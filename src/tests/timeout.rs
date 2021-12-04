mod blocking {
    use crate::{
        tests::utils::{
            clock::MonotonicTestClock,
            op::{high_latency::HighLatencyOp, TestOp},
        },
        traits::IMonotonicClock,
        MonotonicClock,
    };

    use proptest::{prop_assert, prop_assume, proptest};

    use std::time::Duration;

    #[test]
    fn high_latency_op_with_real_clock_returns_within_specified_timeout_plus_small_overhead() {
        let op_latency = Duration::from_millis(1000);
        let timeout = Duration::from_millis(500);
        let small_overhead = Duration::from_millis(100);
        let max_expected_latency = timeout + small_overhead;

        let (actual_latency, _output) = HighLatencyOp::new(MonotonicClock::default(), op_latency)
            .timed_run_with_timeout((), timeout);

        assert!(
            actual_latency <= max_expected_latency,
            "Actual latency of {:?} exceeds expected maximum of {:?}",
            actual_latency,
            max_expected_latency
        );
    }

    proptest! {
        #[test]
        fn high_latency_op_with_test_clock_returns_within_specified_timeout(
            mut op: HighLatencyOp<MonotonicTestClock>,
            timeout: Duration,
        ) {
            // assumptions to skip std::time overflow scenarios
            {
                let now = op.clock().now();

                let after_no_timeout = now.checked_add(op.latency);
                prop_assume!(after_no_timeout.is_some());

                let after_timeout = now.checked_add(timeout);
                prop_assume!(after_timeout.is_some());
            }

            let (actual_latency, _output) = op.timed_run_with_timeout((), timeout);

            prop_assert!(
                actual_latency <= timeout,
                "Actual latency of {:?} exceeds expected timeout of {:?}",
                actual_latency,
                timeout,
            );
        }
    }
}
