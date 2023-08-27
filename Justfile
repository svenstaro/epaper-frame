set dotenv-load := true

default: build

format:
    cargo fmt -- --files-with-diff
    cargo sort --workspace --grouped

lint: clippy docs

clippy $RUST_LOG="info":
    #!/usr/bin/env bash
    set -euxo pipefail

    set -o allexport; source export-esp-rust.sh; set +o allexport
    cargo fmt --check -- --files-with-diff
    cargo sort --workspace --check --grouped
    cargo check --workspace --all-targets
    cargo clippy --workspace --all-targets -- -D warnings

docs $RUSTDOCFLAGS="-D warnings":
    #!/usr/bin/env bash
    set -euo pipefail

    set -o allexport; source export-esp-rust.sh; set +o allexport
    cargo doc --document-private-items

open-docs:
    #!/usr/bin/env bash
    set -euo pipefail

    set -o allexport; source export-esp-rust.sh; set +o allexport
    cargo doc --document-private-items --open

build *args:
    #!/usr/bin/env bash
    set -euo pipefail

    set -o allexport; source export-esp-rust.sh; set +o allexport
    cargo build {{ args }}

run *args:
    #!/usr/bin/env bash
    set -euo pipefail

    set -o allexport; source export-esp-rust.sh; set +o allexport
    cargo espflash flash --monitor {{ args }}

clean:
    rm -rf target .embuild

toolchain:
    #!/usr/bin/env bash
    mkdir -p toolchain
    if (rustup toolchain list | grep -q "esp"); then
        espup update \
            --log-level info \
    else
        espup install \
            --log-level info \
            --targets esp32s3 \
            --export-file export-esp-rust.sh \
            --extended-llvm
    fi
