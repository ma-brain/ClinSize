# Security and Privacy

## Privacy Model

ClinSize should work offline by default. Sample size calculations generally do not require sending data to external services.

Default policy:

- No telemetry by default.
- No external network calls unless a feature clearly requires them.
- No cloud storage in initial versions.
- No patient-level data required for core sample size workflows.

## Local Data

If the app stores recent calculations, settings, or projects:

- Store only what is necessary.
- Make storage location discoverable.
- Provide a way to clear recent items.
- Avoid storing sensitive study details unless the user explicitly saves a project.

## Tauri Security

Use Tauri permissions narrowly:

- Enable only required plugins.
- Restrict filesystem access.
- Avoid unrestricted shell execution.
- Keep command handlers small and reviewed.
- Validate all data crossing the frontend/backend boundary.

## Export Safety

Exported reports should include:

- Software version.
- Calculation method.
- Inputs and outputs.
- Warnings and limitations.

They should not include hidden metadata that exposes local paths or usernames unless required.

## Dependency Management

Review dependencies before adding them:

- Is it maintained?
- Is it necessary?
- Does it introduce native build complexity?
- Does it affect cross-platform packaging?
- Does it handle untrusted input?

Run dependency audits as part of release preparation.

