<script lang="ts">
  import { page } from '$app/stores';
</script>

<div class="error-page">
  <div class="error-content">
    <div class="error-code">{$page.status}</div>
    <h1 class="error-title">
      {#if $page.status === 404}
        Page Not Found
      {:else if $page.status >= 500}
        Server Error
      {:else}
        Something Went Wrong
      {/if}
    </h1>
    <p class="error-message">
      {$page.error?.message || 'An unexpected error occurred.'}
    </p>
    <div class="error-actions">
      <a href="/" class="btn-primary">Go to Dashboard</a>
      <button class="btn-secondary" onclick={() => window.location.reload()}>
        Try Again
      </button>
    </div>
  </div>
</div>

<style>
  .error-page {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: calc(100vh - 200px);
    padding: var(--space-6);
  }

  .error-content {
    text-align: center;
    max-width: 500px;
  }

  .error-code {
    font-size: 6rem;
    font-weight: 800;
    line-height: 1;
    color: var(--color-text-muted);
    margin-bottom: var(--space-4);
    font-family: var(--font-mono);
  }

  .error-title {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-3);
  }

  .error-message {
    color: var(--color-text-secondary);
    margin-bottom: var(--space-6);
    line-height: 1.6;
  }

  .error-actions {
    display: flex;
    gap: var(--space-3);
    justify-content: center;
  }

  .btn-primary {
    background-color: var(--color-accent);
    color: var(--color-background);
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    font-weight: 500;
    text-decoration: none;
    border: none;
    cursor: pointer;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-secondary {
    background-color: transparent;
    color: var(--color-text-secondary);
    padding: var(--space-2) var(--space-4);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    font-weight: 500;
    cursor: pointer;
  }

  .btn-secondary:hover {
    background-color: var(--color-surface);
    color: var(--color-text-primary);
  }
</style>
