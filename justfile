# Matches build-test-native
build:
    cargo build --release
test: build
    cargo test --release
bench:
    cargo bench
lint:
    cargo fmt
    cargo clippy
