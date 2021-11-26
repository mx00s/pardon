# Pardon

Pardon provides:

- [ ] timeout and retry mechanisms for fallible and potentially high-latency operations
- [ ] a pluggable trait-oriented design, so it can be adapted to a variety of contexts and tested without actual sleeps
- [ ] a dynamic policy interface that enables the caller to adapt the retry schedule according to upstream backpressure
- [ ] context about timeouts and retry attempts are returned along with the main operation's eventual result so the caller can react appropriately
- [ ] both sync and async support
- [ ] nostd feature flag
