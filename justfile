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

lint:
    cargo clippy --all-targets --no-deps
    cargo clippy --manifest-path macros/Cargo.toml --all-targets --no-deps

lint-strict:
    cargo clippy --all-targets --no-deps -- -D warnings
    cargo clippy --manifest-path macros/Cargo.toml --all-targets --no-deps -- -D warnings

test:
    cargo nextest run
    cargo nextest run --manifest-path macros/Cargo.toml --no-tests pass
    cargo test --doc
    cargo test --doc --manifest-path macros/Cargo.toml

spell:
    typos

security:
    cargo deny check

pre-push: format-check check lint test spell

quality: pre-push

ci: quality security

package:
    cargo package --manifest-path macros/Cargo.toml
    cargo package
