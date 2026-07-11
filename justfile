# Project command aliases. See handbook `03-development-environment.md`.

# Install all dependencies (Rust workspace + desktop app).
setup:
    cargo fetch
    cd apps/desktop && pnpm install

# Run the desktop app in dev mode.
dev:
    cd apps/desktop && pnpm tauri dev

# Run all tests (Rust workspace + desktop app checks).
test:
    cargo test --workspace
    cd apps/desktop && pnpm check

# Lint and format-check everything.
lint:
    cargo fmt --all -- --check
    cargo clippy --workspace -- -D warnings
    cd apps/desktop && pnpm check

# Build release artifacts.
build:
    cargo build --workspace --release
    cd apps/desktop && pnpm tauri build

# Run the CLI wrapper.
cli *ARGS:
    cargo run -p clinsize-cli -- {{ARGS}}
