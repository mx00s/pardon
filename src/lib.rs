#[cfg(test)]
mod tests;

mod clock;
mod timeout;
mod traits;

pub use {
    clock::MonotonicClock,
    traits::{IDuration, IInstant, IMonotonicClock},
};
