mod blocking {
    use crate::{
        tests::utils::{
            clock::MonotonicTestClock,
            op::{latency::LatencyOp, TestOp},
        },
        MonotonicClock,
    };

    use proptest::{prop_assert_eq, prop_assume, proptest};

    use std::time::{Duration, Instant};

    // Accounts for additional latency for machinery involved in managing an
    // operation's execution.
    const SMALL_OVERHEAD: std::time::Duration = std::time::Duration::from_millis(100);

    #[test]
    fn high_latency_op_with_real_clock_returns_within_specified_timeout_plus_small_overhead() {
        let op_latency = Duration::from_millis(10_000);
        let timeout = Duration::from_millis(500);
        let max_expected_latency = timeout + SMALL_OVERHEAD;

        let (actual_latency, _output) = LatencyOp::new(MonotonicClock::default(), op_latency)
            .timed_run_with_timeout((), timeout);

        assert!(
            actual_latency <= max_expected_latency,
            "Actual latency of {:?} exceeds expected maximum of {:?}",
            actual_latency,
            max_expected_latency
        );
    }

    #[test]
    fn low_latency_op_with_real_clock_returns_faster_than_specified_timeout_plus_small_overhead() {
        let op_latency = Duration::from_millis(500);
        let timeout = Duration::from_millis(10_000);
        let max_expected_latency = op_latency + SMALL_OVERHEAD;

        let (actual_latency, _output) = LatencyOp::new(MonotonicClock::default(), op_latency)
            .timed_run_with_timeout((), timeout);

        assert!(
            actual_latency <= max_expected_latency,
            "Actual latency of {:?} exceeds expected maximum of {:?}",
            actual_latency,
            max_expected_latency
        );
    }

    proptest! {
        // TODO: Change MonotonicTestClock's Instant and Duration types to
        // primitive integer types that can be serialized and shrinked.

        #[test]
        #[ignore]
        fn op_returns_within_min_of_its_latency_and_timeout(
            now: Instant,
            latency: Duration,
            timeout: Duration,
        ) {
            // skip all potential std::time overflow scenarios
            {
                let max_duration = std::cmp::max(latency, timeout);
                prop_assume!(now.checked_add(max_duration).is_some());
            }

            let expected_latency = std::cmp::min(latency, timeout);
            let (actual_latency, _result) = LatencyOp::new(MonotonicTestClock::from(now), latency).timed_run_with_timeout((), timeout);

            prop_assert_eq!(
                expected_latency,
                actual_latency,
                "\n\tExpected latency: {:?}\n\tActual latency: {:?}\n",
                expected_latency,
                actual_latency
            );
        }
    }
}