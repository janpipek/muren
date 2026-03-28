_default:
    @just --list

install:
    cargo install --path .

[group('test')]
test:
    cargo test

# Fix formatting and apply clippy suggestions
[group('dev')]
fix: fmt-fix clippy-fix

# Check formatting and lint (warnings as errors)
[group('lint')]
lint: fmt clippy

[private]
fmt:
    cargo fmt --check

[private]
fmt-fix:
    cargo fmt

[private]
clippy-fix:
    cargo clippy --fix --allow-dirty -- -D warnings

# Build release binary
[group('build')]
build:
    cargo build --release

[private]
clippy:
    cargo clippy -- -D warnings

# Publish to crates.io (dry-run first, then real publish)
[confirm("This will publish to crates.io. Are you sure?")]
[group('maintenance')]
publish: lint test
    cargo publish --dry-run
    cargo publish
