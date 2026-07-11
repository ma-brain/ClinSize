# ClinSize

Cross-platform clinical trial sample size and power desktop app, built with
SvelteKit, Tauri, and Rust. macOS first; Windows and Linux follow.

Engineering, statistical, and AI-collaboration standards live in
[`handbook/`](handbook/README.md) — read that before making architectural or
statistical decisions. Cursor rules derived from it live in `.cursor/rules/`.

## Layout

```text
apps/desktop/       SvelteKit + Tauri application
crates/clinsize-core/  Pure Rust statistical engine (no Tauri/UI deps)
crates/clinsize-cli/   Thin CLI wrapper around clinsize-core
handbook/            Engineering handbook (source of truth)
validation/          Independent validation evidence, per method
examples/            Example inputs, outputs, and reports
```

## Getting started

Requires Rust (via `rustup`), Node.js, and `pnpm`. See
[`handbook/03-development-environment.md`](handbook/03-development-environment.md)
for the full list.

```bash
just setup   # cargo fetch + pnpm install
just dev     # pnpm tauri dev
just test    # cargo test --workspace + pnpm check
just lint    # cargo fmt --check + cargo clippy -D warnings + pnpm check
just build   # release build (Rust + Tauri installer)
just cli list   # list registered methods
just cli calculate --method continuous.two_sample_ttest \
  --input examples/continuous/two-sample-ttest/sample-size.json
```

## Compiling

ClinSize desktop installers are built with Tauri. Build on the target OS for
reliable results — cross-compiling the full desktop app from macOS to Windows
is not supported in this project.

### Prerequisites

All platforms:

- [Rust](https://rustup.rs) (stable toolchain)
- [Node.js 22](https://nodejs.org) (see `.node-version`)
- [pnpm 9](https://pnpm.io) (see `apps/desktop/package.json`)
- [just](https://github.com/casey/just) (optional; wraps the commands below)

Platform-specific:

| Platform | Additional requirements |
| --- | --- |
| macOS | Xcode Command Line Tools |
| Windows | [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with **Desktop development with C++**; WebView2 runtime (included on Windows 10/11) |
| Linux | `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf` (see `.github/workflows/release.yml` for the apt list) |

### Local release build

From the repository root:

```bash
just setup
just build
```

Without `just`:

```bash
cargo fetch
cd apps/desktop && pnpm install
cd apps/desktop && pnpm tauri build
```

Installers are written under:

```text
apps/desktop/src-tauri/target/release/bundle/
```

| Platform | Formats |
| --- | --- |
| macOS | `.app`, `.dmg` |
| Windows | NSIS `.exe` |
| Linux | `.deb`, AppImage |

### Windows from macOS or Linux

To produce a Windows installer without a local Windows machine, use the GitHub
Actions release workflow:

```bash
git tag v0.1.0
git push origin v0.1.0
```

When the **Release** workflow finishes, download the `ClinSize-windows` artifact
from the repository **Actions** tab. The workflow runs:

```bash
cd apps/desktop && pnpm tauri build --target x86_64-pc-windows-msvc
```

on a `windows-latest` runner.

### CLI only (no desktop UI)

The `clinsize` CLI can be cross-compiled without Tauri:

```bash
rustup target add x86_64-pc-windows-msvc   # once per toolchain
cargo build -p clinsize-cli --release --target x86_64-pc-windows-msvc
```

The binary is at `target/x86_64-pc-windows-msvc/release/clinsize.exe`.

More detail: [`handbook/extended-platforms.md`](handbook/extended-platforms.md),
[`handbook/03-development-environment.md`](handbook/03-development-environment.md).

## Status

Phase 6 (extended platforms): Windows and Linux installers via Tauri,
GitHub Actions release builds, and a scriptable `clinsize` CLI are
available. See [`handbook/11-roadmap.md`](handbook/11-roadmap.md) and
[`handbook/extended-platforms.md`](handbook/extended-platforms.md).
