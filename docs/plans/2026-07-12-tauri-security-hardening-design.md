# Tauri Security Hardening Design

**Goal:** Remove renderer-controlled native file paths and make the desktop
application use a restrictive Content Security Policy without removing
user-selected project locations.

## File access

The Tauri backend owns project-file selection. Open and save commands invoke
the native file dialog, accept cancellation as a normal outcome, and then read
or write the selected file. The renderer never supplies a path to a command.

Project files use the `.clinsize.json` extension. Backend validation rejects
any selected path that does not use that extension, preventing accidental use
of unrelated files. Existing project serialization remains unchanged.

## Content Security Policy

Production configuration receives a restrictive CSP suitable for a bundled
Svelte application: resources, scripts, styles, images, and fonts are limited
to the application origin; network connections are disabled; frames and
objects are disallowed. Development continues to use the Vite development
server through the existing Tauri development configuration.

## Validation

Rust tests cover the project-file extension validation and cancellation-safe
command helpers. The full workspace test suite, formatting, Clippy, and
Svelte checks verify the completed change.
