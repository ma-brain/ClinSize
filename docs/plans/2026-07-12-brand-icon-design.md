# Brand Icon Design

## Goal

Show ClinSize's existing application icon beside the product name at the top
of the left navigation rail.

## Chosen approach

Copy the existing `apps/desktop/src-tauri/icons/32x32.png` into the Svelte
application's `static/` directory. The renderer then has a packaged,
build-stable asset without depending on the Tauri source directory at runtime.

## UI behavior

The existing home link will contain a small decorative image and the
`ClinSize` label in a flex row. The icon will be 22–24 px square, retain its
native aspect ratio, and leave the existing subtitle and navigation behavior
unchanged. The image has empty alternative text because the visible label
already supplies the link's accessible name.

## Verification

Use Svelte's type/accessibility checks and inspect the component diff to make
sure the asset path, link behavior, and layout changes are limited to the
brand treatment.
