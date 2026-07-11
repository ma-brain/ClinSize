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
    <h1><a href="/" class:active={$page.url.pathname === "/"}>ClinSize</a></h1>
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
    <section class="nav-section">
      <h2 class="nav-heading">{group.category}</h2>
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
