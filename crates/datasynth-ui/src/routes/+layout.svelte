<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import ConfigSidebar from '$lib/components/config/ConfigSidebar.svelte';
  import { configStore } from '$lib/stores/config';

  let { children } = $props();

  // Determine if we should show sidebar based on route
  let showSidebar = $derived(
    $page.url.pathname.startsWith('/config') ||
    $page.url.pathname === '/' ||
    $page.url.pathname === '/stream'
  );

  // Load config on mount
  onMount(() => {
    configStore.load();
  });

  // Quick save keyboard shortcut
  function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === 's') {
      event.preventDefault();
      configStore.save();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-layout" class:with-sidebar={showSidebar}>
  {#if showSidebar}
    <ConfigSidebar />
  {/if}

  <div class="main-area">
    <main class="main-content">
      {@render children()}
    </main>

    <footer class="app-footer">
      <span class="footer-text">Synthetic Data Generator v0.1.0</span>
      <span class="footer-sep">|</span>
      <span class="footer-shortcut">Ctrl+S to save</span>
    </footer>
  </div>
</div>

<style>
  .app-layout {
    display: flex;
    min-height: 100vh;
  }

  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .main-content {
    flex: 1;
    padding: var(--space-6);
    overflow-y: auto;
  }

  .app-footer {
    border-top: 1px solid var(--color-border);
    padding: var(--space-3) var(--space-6);
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .footer-text {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .footer-sep {
    color: var(--color-border);
  }

  .footer-shortcut {
    font-size: 0.6875rem;
    font-family: var(--font-mono);
    color: var(--color-text-muted);
    background-color: var(--color-surface);
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
  }
</style>
