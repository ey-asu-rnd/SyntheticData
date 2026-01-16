<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import MetricsPanel from '$lib/MetricsPanel.svelte';
  import ControlPanel from '$lib/ControlPanel.svelte';
  import StatusBar from '$lib/StatusBar.svelte';

  interface HealthResponse {
    healthy: boolean;
    version: string;
    uptime_seconds: number;
  }

  interface MetricsResponse {
    total_entries_generated: number;
    total_anomalies_injected: number;
    uptime_seconds: number;
    session_entries: number;
    session_entries_per_second: number;
    active_streams: number;
    total_stream_events: number;
  }

  let connected = $state(false);
  let health: HealthResponse | null = $state(null);
  let metrics: MetricsResponse | null = $state(null);
  let error: string | null = $state(null);
  let loading = $state(true);

  let refreshInterval: ReturnType<typeof setInterval>;

  async function checkHealth() {
    try {
      health = await invoke<HealthResponse>('check_health');
      connected = health.healthy;
      error = null;
    } catch (e) {
      connected = false;
      error = String(e);
    }
  }

  async function fetchMetrics() {
    if (!connected) return;
    try {
      metrics = await invoke<MetricsResponse>('get_metrics');
    } catch (e) {
      console.error('Failed to fetch metrics:', e);
    }
  }

  async function refresh() {
    await checkHealth();
    await fetchMetrics();
  }

  onMount(async () => {
    loading = true;
    await refresh();
    loading = false;

    // Refresh every 2 seconds
    refreshInterval = setInterval(refresh, 2000);
  });

  onDestroy(() => {
    if (refreshInterval) {
      clearInterval(refreshInterval);
    }
  });
</script>

<div class="dashboard">
  <div class="dashboard-header">
    <div>
      <h1>Dashboard</h1>
      <p>Real-time synthetic data generation metrics</p>
    </div>
    <StatusBar {connected} {health} />
  </div>

  {#if loading}
    <div class="loading">
      <p>Connecting to server...</p>
    </div>
  {:else if error && !connected}
    <div class="error-banner">
      <div class="error-content">
        <h3>Server Unavailable</h3>
        <p>Unable to connect to the generation server. Make sure the server is running.</p>
        <code class="mono">{error}</code>
        <button class="btn-primary" onclick={refresh}>Retry Connection</button>
      </div>
    </div>
  {:else}
    <div class="dashboard-grid">
      <div class="main-column">
        <MetricsPanel {metrics} />
      </div>
      <div class="side-column">
        <ControlPanel {connected} />
      </div>
    </div>
  {/if}
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .dashboard-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .dashboard-header h1 {
    margin-bottom: var(--space-1);
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: 1fr 320px;
    gap: var(--space-6);
  }

  .main-column {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .side-column {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  .error-banner {
    background-color: rgba(220, 53, 69, 0.05);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
  }

  .error-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: var(--space-3);
  }

  .error-content h3 {
    color: var(--color-danger);
  }

  .error-content code {
    padding: var(--space-2) var(--space-3);
    background-color: var(--color-surface);
    border-radius: var(--radius-sm);
    max-width: 100%;
    overflow-x: auto;
  }

  @media (max-width: 1024px) {
    .dashboard-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
