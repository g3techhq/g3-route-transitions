set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
    @just --list

setup:
    lefthook install

format:
    cargo fmt --all
    cargo fmt --manifest-path macros/Cargo.toml

format-check:
    cargo fmt --all -- --check
    cargo fmt --manifest-path macros/Cargo.toml -- --check

check:
    cargo check
    cargo check --manifest-path macros/Cargo.toml
    cargo check --target wasm32-unknown-unknown
    cargo check --target aarch64-linux-android --features native-back
    cargo check --target aarch64-apple-ios --features native-back

lint:
    cargo clippy --all-targets --no-deps
    cargo clippy --manifest-path macros/Cargo.toml --all-targets --no-deps

# The web module and the native-back hook compile only for their targets, so
# each is linted there as well as on the host.
lint-strict:
    cargo clippy --all-targets --no-deps -- -D warnings
    cargo clippy --all-targets --no-deps --features native-back -- -D warnings
    cargo clippy --manifest-path macros/Cargo.toml --all-targets --no-deps -- -D warnings
    cargo clippy --no-deps --target wasm32-unknown-unknown -- -D warnings
    cargo clippy --no-deps --target aarch64-linux-android --features native-back -- -D warnings
    cargo clippy --no-deps --target aarch64-apple-ios --features native-back -- -D warnings

test:
    cargo nextest run
    cargo nextest run --manifest-path macros/Cargo.toml --no-tests pass
    cargo test --doc
    cargo test --doc --manifest-path macros/Cargo.toml

spell:
    typos

security:
    cargo deny check

pre-push: format-check check lint-strict test spell

quality: pre-push

package-check:
    cargo package --allow-dirty --manifest-path macros/Cargo.toml
    cargo package --allow-dirty --list

ci: quality security package-check

package:
    cargo package --manifest-path macros/Cargo.toml
    cargo package
