<script lang="ts">
  import type { MethodDescriptor } from "$lib/types";
  import { page } from "$app/stores";

  let { methods }: { methods: MethodDescriptor[] } = $props();

  const workflowLinks = [
    { href: "/project", label: "Project / history" },
    { href: "/scenarios", label: "Scenarios" },
    { href: "/validation", label: "Validation reports" },
  ];

  const categoryOrder = ["Continuous", "Binary", "Count", "Ordinal", "Survival", "Design"];

  // Categories start collapsed on launch; the user expands the ones they need.
  let collapsedCategories = $state(new Set(categoryOrder));

  function toggleCategory(category: string) {
    const next = new Set(collapsedCategories);
    if (next.has(category)) {
      next.delete(category);
    } else {
      next.add(category);
    }
    collapsedCategories = next;
  }

  const groupedMethods = $derived.by(() => {
    const groups = new Map<string, MethodDescriptor[]>();

    for (const method of methods) {
      const category = method.endpointCategory;
      const bucket = groups.get(category) ?? [];
      bucket.push(method);
      groups.set(category, bucket);
    }

    return categoryOrder
      .filter((category) => groups.has(category))
      .map((category) => ({
        category,
        methods: groups.get(category) ?? [],
      }));
  });

  function isActive(href: string, pathname: string): boolean {
    if (href === "/") {
      return pathname === "/";
    }
    return pathname === href || pathname.startsWith(`${href}/`);
  }
</script>

<nav class="rail" aria-label="Application navigation">
  <header class="brand">
    <h1>
      <a href="/" class:active={$page.url.pathname === "/"}>
        <img src="/clinsize-icon.png" alt="" />
        <span>ClinSize</span>
      </a>
    </h1>
    <p>Sample size and power</p>
  </header>

  <section class="nav-section">
    <h2 class="nav-heading">Workflow</h2>
    <ul class="nav-list">
      {#each workflowLinks as link}
        <li>
          <a href={link.href} class:active={isActive(link.href, $page.url.pathname)}>
            {link.label}
          </a>
        </li>
      {/each}
    </ul>
  </section>

  {#each groupedMethods as group}
    {@const collapsed = collapsedCategories.has(group.category)}
    <section class="nav-section">
      <button
        type="button"
        class="nav-heading"
        aria-expanded={!collapsed}
        onclick={() => toggleCategory(group.category)}
      >
        <span>{group.category}</span>
        <span class="chevron" class:open={!collapsed}>▾</span>
      </button>
      {#if !collapsed}
        <ul class="nav-list">
          {#each group.methods as method}
            <li>
              <a
                href="/methods/{method.id}"
                class:active={$page.url.pathname === `/methods/${method.id}`}
              >
                {method.displayName}
              </a>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
  {/each}
</nav>

<style>
  .rail {
    display: flex;
    flex-direction: column;
    gap: 0;
    height: 100vh;
    overflow-y: auto;
    border-right: 1px solid var(--border);
    background: var(--bg-rail);
    padding: 1.25rem 0.85rem 1.5rem;
  }

  .brand {
    margin-bottom: 0.25rem;
    padding-bottom: 0.5rem;
  }

  .brand h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.125rem;
    font-weight: 600;
  }

  .brand h1 a {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .brand h1 img {
    width: 23px;
    height: 23px;
    object-fit: contain;
  }

  .brand p {
    margin: 0.2rem 0 0;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .nav-section {
    padding-top: 0.9rem;
    border-top: 1px solid var(--border);
  }

  .nav-heading {
    margin: 0 0 0.45rem;
    padding: 0 0.55rem;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-primary);
    opacity: 0.62;
  }

  button.nav-heading {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font: inherit;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    background: none;
    border: none;
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: opacity var(--transition-fast);
  }

  button.nav-heading:hover {
    opacity: 1;
  }

  button.nav-heading:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .chevron {
    font-size: 0.65rem;
    transition: transform var(--transition-fast);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .nav-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 0.15rem;
  }

  .nav-list a {
    display: block;
    padding: 0.45rem 0.55rem;
    border-radius: var(--radius-sm);
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-primary);
    transition:
      background var(--transition-fast),
      color var(--transition-fast);
  }

  .nav-list a:hover {
    background: var(--bg-subtle);
    color: var(--accent);
  }

  .nav-list a.active {
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .brand a.active {
    color: var(--accent);
  }
</style>
