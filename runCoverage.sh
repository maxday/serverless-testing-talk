mkdir -p target/coverage/html
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='target/cargo-test-%p-%m.profraw' cargo test --bins
grcov . -s src \
--binary-path ./target/debug/ -t html \
--branch --ignore-not-existing -o ./target/debug/coverage/
open target/debug/coverage/index.html