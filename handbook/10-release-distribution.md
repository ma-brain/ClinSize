# Release and Distribution

## Supported Platforms

Initial supported platforms:

- macOS.
- Windows.
- Linux, if packaging effort is acceptable.

Tauri can target all three, but installer generation, signing, and update infrastructure are platform-specific.

## Versioning

Use semantic versioning:

- Major: breaking project format or method behavior changes.
- Minor: new methods or significant features.
- Patch: bug fixes and documentation corrections.

Statistical method changes that alter results must be documented clearly in release notes.

## Release Checklist

Before release:

- All tests pass.
- Validation reports are updated.
- Method documentation matches implementation.
- Release notes list new methods and known limitations.
- Installers build on target platforms.
- macOS app is signed and notarized if distributed publicly.
- Windows installer is signed if distributed publicly.

## Result Reproducibility

Every exported calculation should include:

- App version.
- Core engine version.
- Method identifier.
- Method documentation version.
- Input parameters.
- Result.
- Date/time generated.

This allows users to understand whether a calculation can be reproduced later.

## Auto-Updates

Auto-updates can be added after stable signing and release infrastructure exists. Do not add auto-update before the release process is understood.

## Regulatory Language

Avoid marketing language that implies regulatory validation unless formal validation evidence exists. Use clear wording such as:

"This software provides documented statistical calculations. Users are responsible for confirming suitability for their regulatory and organizational requirements."

