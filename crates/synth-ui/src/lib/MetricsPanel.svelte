<script lang="ts">
  interface MetricsResponse {
    total_entries_generated: number;
    total_anomalies_injected: number;
    uptime_seconds: number;
    session_entries: number;
    session_entries_per_second: number;
    active_streams: number;
    total_stream_events: number;
  }

  let { metrics }: { metrics: MetricsResponse | null } = $props();

  function formatNumber(num: number): string {
    if (num >= 1_000_000) {
      return (num / 1_000_000).toFixed(2) + 'M';
    } else if (num >= 1_000) {
      return (num / 1_000).toFixed(1) + 'K';
    }
    return num.toLocaleString();
  }

  function formatRate(rate: number): string {
    return rate.toFixed(1);
  }

  function getAnomalyRate(): string {
    if (!metrics || metrics.total_entries_generated === 0) return '0.0';
    return ((metrics.total_anomalies_injected / metrics.total_entries_generated) * 100).toFixed(2);
  }
</script>

<section class="metrics-panel">
  <header class="panel-header">
    <h2>Generation Metrics</h2>
    <span class="badge">Live</span>
  </header>

  {#if metrics}
    <div class="metrics-grid">
      <div class="metric-card highlight">
        <div class="metric-value mono">{formatNumber(metrics.total_entries_generated)}</div>
        <div class="metric-label">Total Entries</div>
      </div>

      <div class="metric-card">
        <div class="metric-value mono">{formatRate(metrics.session_entries_per_second)}</div>
        <div class="metric-label">Entries / Second</div>
      </div>

      <div class="metric-card">
        <div class="metric-value mono">{formatNumber(metrics.total_anomalies_injected)}</div>
        <div class="metric-label">Anomalies Injected</div>
      </div>

      <div class="metric-card">
        <div class="metric-value mono">{getAnomalyRate()}%</div>
        <div class="metric-label">Anomaly Rate</div>
      </div>
    </div>

    <div class="divider"></div>

    <div class="secondary-metrics">
      <div class="secondary-metric">
        <span class="secondary-label">Active Streams</span>
        <span class="secondary-value mono">{metrics.active_streams}</span>
      </div>
      <div class="secondary-metric">
        <span class="secondary-label">Stream Events</span>
        <span class="secondary-value mono">{formatNumber(metrics.total_stream_events)}</span>
      </div>
      <div class="secondary-metric">
        <span class="secondary-label">Session Entries</span>
        <span class="secondary-value mono">{formatNumber(metrics.session_entries)}</span>
      </div>
    </div>
  {:else}
    <div class="no-data">
      <p>No metrics available</p>
    </div>
  {/if}
</section>

<style>
  .metrics-panel {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-5);
  }

  .panel-header h2 {
    font-size: 1rem;
    font-weight: 600;
  }

  .metrics-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-4);
  }

  .metric-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    text-align: center;
  }

  .metric-card.highlight {
    border-color: var(--color-accent);
    background-color: rgba(0, 102, 255, 0.02);
  }

  .metric-value {
    font-size: 1.75rem;
    font-weight: 600;
    color: var(--color-text-primary);
    line-height: 1.2;
    margin-bottom: var(--space-1);
  }

  .metric-card.highlight .metric-value {
    color: var(--color-accent);
  }

  .metric-label {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .secondary-metrics {
    display: flex;
    gap: var(--space-6);
  }

  .secondary-metric {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .secondary-label {
    font-size: 0.75rem;
    color: var(--color-text-muted);
  }

  .secondary-value {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .no-data {
    padding: var(--space-6);
    text-align: center;
    color: var(--color-text-muted);
  }

  @media (max-width: 768px) {
    .metrics-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .secondary-metrics {
      flex-direction: column;
      gap: var(--space-3);
    }
  }
</style>
