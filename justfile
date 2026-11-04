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

check: && check-ios
    cargo check
    cargo check --manifest-path macros/Cargo.toml
    cargo check --target wasm32-unknown-unknown
    cargo check --target aarch64-linux-android --features native-back

# Dioxus's asset crate pulls in objc2 on iOS, whose build script compiles C
# through Xcode's xcrun, so only a Mac can build that target. The apps' mobile
# workflows build the native-back hook for iOS on macOS.
[macos]
check-ios:
    cargo check --target aarch64-apple-ios --features native-back

[linux]
[windows]
check-ios:
    @echo "Skipping aarch64-apple-ios: it needs Xcode."

lint:
    cargo clippy --all-targets --no-deps
    cargo clippy --manifest-path macros/Cargo.toml --all-targets --no-deps

# The web module and the native-back hook compile only for their targets, so
# each is linted there as well as on the host.
lint-strict: && lint-strict-ios
    cargo clippy --all-targets --no-deps -- -D warnings
    cargo clippy --all-targets --no-deps --features native-back -- -D warnings
    cargo clippy --all-targets --no-deps --features timing -- -D warnings
    cargo clippy --manifest-path macros/Cargo.toml --all-targets --no-deps -- -D warnings
    cargo clippy --no-deps --target wasm32-unknown-unknown -- -D warnings
    cargo clippy --no-deps --target aarch64-linux-android --features native-back -- -D warnings

[macos]
lint-strict-ios:
    cargo clippy --no-deps --target aarch64-apple-ios --features native-back -- -D warnings

[linux]
[windows]
lint-strict-ios:
    @echo "Skipping aarch64-apple-ios: it needs Xcode."

test:
    cargo nextest run
    cargo nextest run --features timing
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

# Readability rules inside RSX, checked without rewriting source.
style-check:
    node scripts/rust-style.mjs

style-report:
    node scripts/rust-style.mjs --report

style-test:
    node --test scripts/rust-style.test.mjs
