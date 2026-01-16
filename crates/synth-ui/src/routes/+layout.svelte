<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';

  let { children } = $props();

  // Navigation links configuration
  const navLinks = [
    { href: '/', label: 'Dashboard' },
    { href: '/config', label: 'Configuration' },
    { href: '/stream', label: 'Stream' },
  ];

  // Check if a nav link is active based on current path
  function isActive(href: string, currentPath: string): boolean {
    if (href === '/') {
      return currentPath === '/';
    }
    return currentPath.startsWith(href);
  }
</script>

<div class="app-container">
  <header class="app-header">
    <div class="header-content">
      <div class="logo">
        <span class="logo-text">SYNTH</span>
        <span class="logo-subtext">Data Generator</span>
      </div>
      <nav class="nav">
        {#each navLinks as link}
          <a
            href={link.href}
            class="nav-link"
            class:active={isActive(link.href, $page.url.pathname)}
          >
            {link.label}
          </a>
        {/each}
      </nav>
    </div>
  </header>

  <main class="app-main">
    {@render children()}
  </main>

  <footer class="app-footer">
    <span class="footer-text">Synthetic Data Generator v0.1.0</span>
  </footer>
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .app-header {
    border-bottom: 1px solid var(--color-border);
    background-color: var(--color-background);
    position: sticky;
    top: 0;
    z-index: 100;
  }

  .header-content {
    max-width: 1400px;
    margin: 0 auto;
    padding: var(--space-4) var(--space-6);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .logo {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }

  .logo-text {
    font-size: 1.25rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--color-text-primary);
  }

  .logo-subtext {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .nav {
    display: flex;
    gap: var(--space-5);
  }

  .nav-link {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-decoration: none;
    padding: var(--space-2) 0;
    border-bottom: 2px solid transparent;
    transition: all var(--transition-fast);
  }

  .nav-link:hover {
    color: var(--color-text-primary);
  }

  .nav-link.active {
    color: var(--color-text-primary);
    border-bottom-color: var(--color-accent);
  }

  .app-main {
    flex: 1;
    max-width: 1400px;
    width: 100%;
    margin: 0 auto;
    padding: var(--space-6);
  }

  .app-footer {
    border-top: 1px solid var(--color-border);
    padding: var(--space-4) var(--space-6);
    text-align: center;
  }

  .footer-text {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }
</style>
