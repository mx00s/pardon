/// Intantenous moment in time.
///
/// Conceptually, this is a point on a timeline.
pub trait IInstant: Clone + Eq + Ord {
    /// Preferred `IDuration` type to use in conjunction with this instant type.
    type Duration: IDuration<Instant = Self>;

    fn checked_duration_since(&self, earlier: Self) -> Option<Self::Duration>;

    fn checked_add(&self, duration: Self::Duration) -> Option<Self>;
}

/// Measure of elapsed time.
///
/// Conceptually, this is the distance between two points on a timeline and
/// should never be negative.
pub trait IDuration: Clone + Eq + Ord + Sized {
    /// Preferred `IInstant` type to use in conjunction with this duration type.
    type Instant: IInstant<Duration = Self>;
}

/// Monotonically non-decreasing clock.
///
/// The monotonicity contract means that when two samples are taken in succession
/// of the current clock's time the second sample cannot be in the past relative to
/// the first sample.
pub trait IMonotonicClock {
    type Instant: IInstant<Duration = Self::Duration>;
    type Duration: IDuration<Instant = Self::Instant>;

    /// Returns current instant according to this clock.
    fn now(&self) -> Self::Instant;

    /// Sleeps for the specified duration of time.
    ///
    /// Implementations for testing purposes are not expected to actually sleep.
    fn sleep(&mut self, duration: &Self::Duration);
}

// TODO: add macro to generate proptest that tries to find a counterexample
// to an `IMonotonicClock` impl's non-decreasing monotonicity contract.
