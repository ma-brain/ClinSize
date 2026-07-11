# Extended Platforms

Phase 6 extends ClinSize beyond the macOS-first desktop workflow.

## Desktop Installers

`pnpm tauri build` (via `just build`) produces platform installers when run
on the target OS:

| Platform | Formats | Output location |
| --- | --- | --- |
| macOS | `.app`, `.dmg` | `apps/desktop/src-tauri/target/release/bundle/` |
| Windows | NSIS `.exe` | same |
| Linux | `.deb`, AppImage | same |

Tauri bundle settings live in `apps/desktop/src-tauri/tauri.conf.json`.
Signing and notarization are platform-specific and documented in
`10-release-distribution.md`.

Tagged releases (`v*`) trigger `.github/workflows/release.yml`, which builds
installers on macOS, Windows, and Linux runners and uploads them as CI
artifacts.

## CLI

The `clinsize` binary wraps `clinsize-core` for scripting and validation:

```bash
just cli list
just cli version
just cli calculate --method continuous.two_sample_ttest \
  --input examples/continuous/two-sample-ttest/sample-size.json
just cli report --method continuous.two_sample_ttest \
  --input examples/continuous/two-sample-ttest/sample-size.json \
  --result /tmp/result.json
just cli validation-report --method continuous.two_sample_ttest
```

Inputs and results are JSON documents using the same camelCase field names as
the desktop app and Tauri commands. Method routing is implemented in
`clinsize-core::dispatch` so future bindings can reuse it.

## Future Web or Mobile Prototypes

A web or mobile prototype should consume the same `clinsize-core` engine,
either through:

- WebAssembly compilation of the core crate, or
- A thin HTTP/JSON service that calls `dispatch::calculate_json`.

No web or mobile shell is implemented yet. Keep UI logic out of the core
crate when adding one.
