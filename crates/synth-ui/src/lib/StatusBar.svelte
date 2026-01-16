<script lang="ts">
  interface HealthResponse {
    healthy: boolean;
    version: string;
    uptime_seconds: number;
  }

  let { connected, health }: { connected: boolean; health: HealthResponse | null } = $props();

  function formatUptime(seconds: number): string {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    }
    return `${secs}s`;
  }
</script>

<div class="status-bar">
  <div class="status-item">
    <span class="status-dot" class:active={connected} class:inactive={!connected}></span>
    <span class="status-text">{connected ? 'Connected' : 'Disconnected'}</span>
  </div>

  {#if connected && health}
    <div class="divider-vertical"></div>
    <div class="status-item">
      <span class="status-label">Version</span>
      <span class="status-value mono">{health.version}</span>
    </div>
    <div class="divider-vertical"></div>
    <div class="status-item">
      <span class="status-label">Uptime</span>
      <span class="status-value mono">{formatUptime(health.uptime_seconds)}</span>
    </div>
  {/if}
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-2) var(--space-4);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .status-text {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .status-label {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .status-value {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .divider-vertical {
    width: 1px;
    height: 16px;
    background-color: var(--color-border);
  }
</style>
