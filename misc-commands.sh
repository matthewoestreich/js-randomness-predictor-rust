# To report on test execution times
cargo +nightly test --tests -- -Zunstable-options --report-time

# Format entire project
cargo fmt

# Allow for `println!` to show in tests
cargo test -- --nocapture