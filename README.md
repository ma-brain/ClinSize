# ClinSize

Sample size, power, and design calculations for clinical trials.

ClinSize is a desktop workbench and a command-line tool. Both call the same
Rust engine, `clinsize-core`. The app runs offline on macOS, Windows, and
Linux. Nothing leaves the machine.

It is for statisticians, trial methodologists, and study teams who need
reproducible numbers with the assumptions printed next to the result.

## Features

- Twenty-one methods across continuous, binary, count, ordinal, survival,
  and design problems.
- Sample size and power solve modes on the endpoint methods.
- One-parameter sensitivity sweeps with a chart on each method screen.
- Local project files (`.clinsize.json`) with calculation history.
- Side-by-side scenario comparison.
- Export to Markdown, HTML, Word-compatible HTML, and printable PDF.
- A scriptable `clinsize` CLI for batch work and CI.
- Independent validation cases in [`validation/`](validation/), checked in
  CI against published R references.

ClinSize is not a certified system for regulatory submission. Judge fitness
for a given study from the cases and sources under `validation/`.

## Methods

| Category | Method | Identifier |
| --- | --- | --- |
| Continuous | One-sample t-test | `continuous.one_sample_ttest` |
| Continuous | Two-sample t-test | `continuous.two_sample_ttest` |
| Continuous | Paired t-test | `continuous.paired_ttest` |
| Continuous | One-way ANOVA | `continuous.one_way_anova` |
| Continuous | Two-way ANOVA | `continuous.two_way_anova` |
| Continuous | Two-sample ANCOVA | `continuous.ancova_two_sample` |
| Continuous | Change from baseline | `continuous.change_from_baseline` |
| Continuous | MMRM (longitudinal) | `continuous.mmrm` |
| Continuous | Mann-Whitney U | `continuous.mann_whitney` |
| Continuous | Wilcoxon signed-rank | `continuous.wilcoxon_signed_rank` |
| Binary | Difference in proportions | `binary.two_proportion_difference` |
| Binary | Odds ratio | `binary.odds_ratio` |
| Binary | One-sample binomial | `binary.one_sample_binomial` |
| Binary | Risk ratio | `binary.risk_ratio` |
| Count | Negative binomial | `count.negative_binomial` |
| Count | Poisson | `count.poisson` |
| Ordinal | Proportional odds | `ordinal.proportional_odds` |
| Survival | Log-rank test | `survival.log_rank` |
| Design | Multiplicity adjustment | `design.multiplicity` |
| Design | Group sequential design | `design.group_sequential` |
| Design | Blinded sample size re-estimation | `design.blinded_ssre` |

## Install

Build from source on the machine you will run. Cross-compiling the desktop
app is not supported.

Tagged releases (`v*`) build macOS (Apple Silicon and Intel), Windows
(NSIS), and Linux (`.deb` and AppImage) installers and attach them to the
GitHub release. Download the `.dmg` for your Mac from
[Releases](https://github.com/ma-brain/ClinSize/releases).

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
| Windows | [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with Desktop development with C++. WebView2 is included on Windows 10/11. |
| Linux | `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf` |

### Desktop app

```bash
just setup   # cargo fetch + pnpm install
just dev     # pnpm tauri dev
```

Without `just`:

```bash
cargo fetch
cd apps/desktop && pnpm install && pnpm tauri dev
```

### Command line

```bash
just cli list
just cli calculate --method continuous.two_sample_ttest \
  --input examples/continuous/two-sample-ttest/sample-size.json \
  --output result.json
just cli report --method continuous.two_sample_ttest \
  --input examples/continuous/two-sample-ttest/sample-size.json \
  --result result.json
just cli validation-report --method continuous.two_sample_ttest
```

Without `just`, use `cargo run -p clinsize-cli --` in place of `just cli`.
More example inputs live in [`examples/`](examples/).

### Release installers

From the repository root, on the target OS:

```bash
just setup
just build
```

Without `just`:

```bash
cargo fetch
cd apps/desktop && pnpm install && pnpm tauri build
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

To produce installers without a matching local machine, push a version tag
and download the files from the GitHub release the workflow updates:

```bash
git tag v0.1.0
git push origin v0.1.0
```

A failed tag run can be retried without moving the tag: run the Release
workflow with `workflow_dispatch` and the existing tag, or push a
`v<version>-rebuild` tag. Both check out that version and attach
installers to its GitHub release. macOS jobs run
`pnpm tauri build --target aarch64-apple-darwin` and
`--target x86_64-apple-darwin` on `macos-latest`.

The `clinsize` CLI can be cross-compiled without Tauri:

```bash
rustup target add x86_64-pc-windows-msvc   # once per toolchain
cargo build -p clinsize-cli --release --target x86_64-pc-windows-msvc
```

The binary lands at `target/x86_64-pc-windows-msvc/release/clinsize.exe`.

## Validation

Every method has a folder under [`validation/`](validation/) with:

- `cases.json`. Inputs and expected values, re-run through the engine in CI.
- `reference-output.md`. The R commands or published tables those values
  came from.

Expected values come from the external reference, never from the engine
itself. Current sources include Noether (1987), Zhu and Lakkis (2014),
Whitehead (1993), Signorini (1991), and the R functions `power.t.test`,
`gsDesign`, `longpower`, `EnvStats`, and `Hmisc`.

Generate a report for one method:

```bash
just cli validation-report --method continuous.two_sample_ttest
```

The desktop app has a Validation page that shows the same evidence.

## Repository layout

```text
apps/desktop/          SvelteKit + Tauri desktop app
crates/clinsize-core/  Statistical engine (no UI or Tauri deps)
crates/clinsize-cli/   clinsize CLI
validation/            Per-method reference cases
examples/              Example JSON inputs
```

Statistical formulas live in `clinsize-core`. The UI and CLI never
implement them.

## Development

```bash
just test    # cargo test --workspace + pnpm check
just lint    # rustfmt, clippy -D warnings, pnpm check
just build   # release build (Rust + Tauri installer)
```
