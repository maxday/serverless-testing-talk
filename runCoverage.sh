mkdir -p target/coverage/html
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='./target/cargo-test-%p-%m.profraw' cargo test --bins
grcov ./target --binary-path ./target/debug/deps/ -s ./src -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html
open target/coverage/html/index.html