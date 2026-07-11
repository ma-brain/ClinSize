# Development Environment

## Required Tools On macOS

Install:

- Xcode or Xcode Command Line Tools.
- Homebrew.
- Rust through `rustup`.
- Node.js through `fnm`, `nvm`, or another version manager.
- `pnpm`.
- Cursor.
- Git.

Recommended optional tools:

- `just` for project command aliases.
- `cargo-nextest` for faster Rust test execution.
- `cargo-watch` for watch mode.
- `bacon` for continuous Rust feedback.
- SQLite or DuckDB if local persistence is added.

## Verification Commands

The project should eventually provide these commands:

```bash
pnpm install
pnpm check
pnpm test
pnpm tauri dev
pnpm tauri build
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all --check
```

Prefer a `justfile` so common workflows are discoverable:

```bash
just setup
just dev
just test
just lint
just build
```

## Cursor Extensions

Install:

- rust-analyzer.
- Svelte.
- Even Better TOML.
- Prettier.
- Error Lens.
- ESLint, if the frontend enables ESLint.

## Local Development Workflow

Typical desktop workflow:

```bash
pnpm install
pnpm tauri dev
```

Typical Rust engine workflow:

```bash
cargo test -p clinsize-core
cargo clippy -p clinsize-core -- -D warnings
```

Typical UI workflow:

```bash
pnpm check
pnpm test
```

## Version Pinning

Pin major tool versions where possible:

- Rust edition in `Cargo.toml`.
- Node version in `.nvmrc` or `.node-version`.
- Package manager in `package.json`.
- Tauri major version.
- SvelteKit major version.

This reduces drift between macOS, Windows, Linux, and CI.

