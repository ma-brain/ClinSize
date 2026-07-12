# Brand Icon Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add the existing ClinSize app icon beside the product name in the navigation rail.

**Architecture:** Copy the 32 px Tauri icon into the Svelte `static/` directory and reference it from the existing brand home link. Keep navigation state and branding text unchanged; CSS handles the horizontal icon/label alignment.

**Tech Stack:** Svelte 5, CSS, Tauri-packaged PNG asset.

---

### Task 1: Package and render the existing icon

**Files:**
- Create: `apps/desktop/static/clinsize-icon.png`
- Modify: `apps/desktop/src/lib/components/NavRail.svelte:55-61,115-132`

**Step 1: Write the failing asset/markup check**

Run a shell assertion that requires the frontend asset and an image within the brand link:

`test -f apps/desktop/static/clinsize-icon.png && grep -q 'src="/clinsize-icon.png"' apps/desktop/src/lib/components/NavRail.svelte`

**Step 2: Run it to prove it fails**

Expected: failure because the frontend asset and image markup do not exist.

**Step 3: Implement the minimal UI change**

Copy `apps/desktop/src-tauri/icons/32x32.png` to the frontend static path. Wrap the existing `ClinSize` text in a flex label within the existing home link, add a decorative image with empty `alt`, and add compact 22–24 px brand-icon styles. Preserve the title, subtitle, route, and active-link state.

**Step 4: Verify the check and Svelte component**

Run the assertion, `pnpm --dir apps/desktop check`, and `git diff --check`. Expected: the assertion passes, Svelte check has zero errors, and the diff check passes.

**Step 5: Commit**

Stage the asset and `NavRail.svelte`, then commit with message `feat: add icon to navigation brand`.
