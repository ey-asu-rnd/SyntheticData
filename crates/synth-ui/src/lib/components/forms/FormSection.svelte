<script lang="ts">
  import type { Snippet } from 'svelte';

  /**
   * Collapsible form section with title and optional description.
   */
  let {
    title,
    description = '',
    collapsible = false,
    collapsed = false,
    children,
  }: {
    title: string;
    description?: string;
    collapsible?: boolean;
    collapsed?: boolean;
    children: Snippet;
  } = $props();

  let isCollapsed = $state(false);

  // Sync with prop when it changes
  $effect(() => {
    isCollapsed = collapsed;
  });

  function toggle() {
    if (collapsible) {
      isCollapsed = !isCollapsed;
    }
  }
</script>

<section class="form-section">
  <button
    type="button"
    class="section-header"
    class:collapsible
    onclick={toggle}
    disabled={!collapsible}
  >
    <div class="section-title-group">
      <h3 class="section-title">{title}</h3>
      {#if description}
        <p class="section-description">{description}</p>
      {/if}
    </div>
    {#if collapsible}
      <span class="collapse-icon" class:collapsed={isCollapsed}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 9l6 6 6-6" />
        </svg>
      </span>
    {/if}
  </button>

  {#if !isCollapsed}
    <div class="section-content">
      {@render children()}
    </div>
  {/if}
</section>

<style>
  .form-section {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .section-header {
    width: 100%;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: var(--space-4) var(--space-5);
    background: none;
    border: none;
    text-align: left;
    cursor: default;
    border-radius: 0;
  }

  .section-header.collapsible {
    cursor: pointer;
  }

  .section-header.collapsible:hover {
    background-color: var(--color-background);
  }

  .section-header:disabled {
    cursor: default;
  }

  .section-title-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .section-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .section-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .collapse-icon {
    width: 20px;
    height: 20px;
    color: var(--color-text-muted);
    transition: transform var(--transition-fast);
    flex-shrink: 0;
    margin-top: var(--space-1);
  }

  .collapse-icon.collapsed {
    transform: rotate(-90deg);
  }

  .collapse-icon svg {
    width: 100%;
    height: 100%;
  }

  .section-content {
    padding: 0 var(--space-5) var(--space-5);
    border-top: 1px solid var(--color-border);
    padding-top: var(--space-4);
  }
</style>
