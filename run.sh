set -euxo pipefail

cargo fix --allow-dirty --allow-staged && cargo fmt
cargo clippy
cargo check
cargo nextest run