# To report on test execution times
cargo +nightly test --tests -- -Zunstable-options --report-time

# Format entire project
cargo fmt

# Allow for `println!` to show in tests
cargo test -- --nocapture

# Target specific test
cargo test --package jsrp --lib -- node_predictor::tests::general_tests


###########################
# For testing the CLI
###########################
jsrp firefox -s 0.983788222968869 0.6210323993153665 0.37646090421893474 0.13923801694587312