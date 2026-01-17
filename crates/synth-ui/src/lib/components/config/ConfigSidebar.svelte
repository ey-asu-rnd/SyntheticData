<script lang="ts">
  import { page } from '$app/stores';
  import { configStore } from '$lib/stores/config';

  // Navigation structure
  const navItems = [
    {
      section: 'Generate',
      items: [
        { href: '/', label: 'Dashboard', icon: 'dashboard' },
        { href: '/stream', label: 'Stream Viewer', icon: 'stream' },
      ],
    },
    {
      section: 'Configuration',
      items: [
        { href: '/config', label: 'Overview', icon: 'settings' },
        { href: '/config/global', label: 'Global Settings', icon: 'globe' },
        { href: '/config/companies', label: 'Companies', icon: 'building' },
        { href: '/config/transactions', label: 'Transactions', icon: 'chart' },
        { href: '/config/output', label: 'Output', icon: 'export' },
      ],
    },
    {
      section: 'Master Data',
      items: [
        { href: '/config/master-data', label: 'Overview', icon: 'database' },
        { href: '/config/master-data/vendors', label: 'Vendors', icon: 'vendor' },
        { href: '/config/master-data/customers', label: 'Customers', icon: 'customer' },
        { href: '/config/master-data/materials', label: 'Materials', icon: 'material' },
        { href: '/config/master-data/assets', label: 'Fixed Assets', icon: 'asset' },
      ],
    },
    {
      section: 'Document Flows',
      items: [
        { href: '/config/document-flows', label: 'Overview', icon: 'flow' },
        { href: '/config/document-flows/p2p', label: 'Procure to Pay', icon: 'p2p' },
        { href: '/config/document-flows/o2c', label: 'Order to Cash', icon: 'o2c' },
      ],
    },
    {
      section: 'Advanced',
      items: [
        { href: '/config/financial', label: 'Financial', icon: 'dollar' },
        { href: '/config/compliance', label: 'Fraud & Controls', icon: 'shield' },
        { href: '/config/analytics', label: 'Analytics', icon: 'analytics' },
      ],
    },
    {
      section: 'Presets',
      items: [
        { href: '/presets', label: 'Manage Presets', icon: 'preset' },
      ],
    },
  ];

  let { collapsed = false } = $props();

  function isActive(href: string, currentPath: string): boolean {
    if (href === '/') {
      return currentPath === '/';
    }
    if (href === '/config') {
      return currentPath === '/config';
    }
    return currentPath.startsWith(href);
  }

  // Get icon SVG (simple inline icons)
  function getIcon(name: string): string {
    const icons: Record<string, string> = {
      dashboard: 'M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z',
      stream: 'M4 6h16M4 12h16M4 18h16',
      settings: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z',
      globe: 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z',
      building: 'M6 22V4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v18M6 22h12M10 6h.01M14 6h.01M10 10h.01M14 10h.01M10 14h.01M14 14h.01M10 18h.01M14 18h.01',
      chart: 'M18 20V10M12 20V4M6 20v-6',
      export: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12',
      database: 'M21 5c0 1.1-4 2-9 2s-9-.9-9-2m18 0c0-1.1-4-2-9-2s-9 .9-9 2m18 0v14c0 1.1-4 2-9 2s-9-.9-9-2V5m18 7c0 1.1-4 2-9 2s-9-.9-9-2',
      vendor: 'M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M12 7a4 4 0 1 0-8 0 4 4 0 0 0 8 0zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
      customer: 'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 7a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
      material: 'M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16zM3.27 6.96L12 12.01l8.73-5.05M12 22.08V12',
      asset: 'M2 20h20M6 16V8a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v8M9 16v-6h6v6',
      flow: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5',
      p2p: 'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v0a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2zM9 12h6M9 16h6',
      o2c: 'M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1zM12 11v6M9 14l3 3 3-3',
      dollar: 'M12 1v22M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6',
      shield: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10zM9 12l2 2 4-4',
      analytics: 'M21 21H4.6c-.6 0-1-.4-1-1V3M7 14l4-4 4 4 6-6',
      preset: 'M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2zM17 21v-8H7v8M7 3v5h8',
    };
    return icons[name] || 'M12 12m-10 0a10 10 0 1 0 20 0 10 10 0 1 0-20 0';
  }

  const isDirty = configStore.isDirty;
  const isValid = configStore.isValid;
</script>

<aside class="sidebar" class:collapsed>
  <div class="sidebar-header">
    {#if !collapsed}
      <span class="logo-text">SYNTH</span>
      <span class="logo-subtext">Configuration</span>
    {:else}
      <span class="logo-text-short">S</span>
    {/if}
  </div>

  <nav class="sidebar-nav">
    {#each navItems as group}
      <div class="nav-group">
        {#if !collapsed}
          <div class="nav-group-label">{group.section}</div>
        {/if}
        {#each group.items as item}
          <a
            href={item.href}
            class="nav-item"
            class:active={isActive(item.href, $page.url.pathname)}
            title={collapsed ? item.label : ''}
          >
            <svg
              class="nav-icon"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d={getIcon(item.icon)} />
            </svg>
            {#if !collapsed}
              <span class="nav-label">{item.label}</span>
            {/if}
          </a>
        {/each}
      </div>
    {/each}
  </nav>

  <div class="sidebar-footer">
    {#if $isDirty}
      <div class="status-indicator" class:warning={!$isValid}>
        {#if !collapsed}
          <span class="status-dot" class:warning={!$isValid} class:active={$isValid}></span>
          <span>Unsaved changes</span>
        {:else}
          <span class="status-dot" class:warning={!$isValid} class:active={$isValid}></span>
        {/if}
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 240px;
    min-width: 240px;
    height: 100vh;
    background-color: var(--color-surface);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    position: sticky;
    top: 0;
    overflow-y: auto;
    transition: width 200ms ease, min-width 200ms ease;
  }

  .sidebar.collapsed {
    width: 60px;
    min-width: 60px;
  }

  .sidebar-header {
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-1);
  }

  .collapsed .sidebar-header {
    align-items: center;
  }

  .logo-text {
    font-size: 1rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--color-text-primary);
  }

  .logo-text-short {
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-accent);
  }

  .logo-subtext {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .sidebar-nav {
    flex: 1;
    padding: var(--space-3);
  }

  .nav-group {
    margin-bottom: var(--space-4);
  }

  .nav-group-label {
    font-size: 0.625rem;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    padding: var(--space-2) var(--space-2);
    margin-bottom: var(--space-1);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    border-radius: var(--radius-md);
    color: var(--color-text-secondary);
    text-decoration: none;
    font-size: 0.8125rem;
    font-weight: 500;
    transition: all var(--transition-fast);
  }

  .collapsed .nav-item {
    justify-content: center;
    padding: var(--space-2);
  }

  .nav-item:hover {
    background-color: var(--color-background);
    color: var(--color-text-primary);
  }

  .nav-item.active {
    background-color: var(--color-accent);
    color: white;
  }

  .nav-icon {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }

  .nav-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sidebar-footer {
    padding: var(--space-3);
    border-top: 1px solid var(--color-border);
    min-height: 48px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .collapsed .status-indicator {
    justify-content: center;
  }

  .status-indicator.warning {
    color: var(--color-warning);
  }
</style>
