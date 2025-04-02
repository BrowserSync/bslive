set -euxo pipefail

cargo fix --allow-dirty --allow-staged && cargo fmt
cargo clippy
cargo check
bash gen.sh
cargo nextest run