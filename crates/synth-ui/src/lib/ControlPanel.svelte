<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface StreamResponse {
    success: boolean;
    message: string;
  }

  interface BulkGenerateResponse {
    success: boolean;
    entries_generated: number;
    duration_ms: number;
    anomaly_count: number;
  }

  let { connected }: { connected: boolean } = $props();

  let streamStatus = $state<'idle' | 'running' | 'paused'>('idle');
  let generating = $state(false);
  let lastResult: BulkGenerateResponse | null = $state(null);
  let error: string | null = $state(null);

  // Bulk generation options
  let entryCount = $state(1000);
  let includeMasterData = $state(false);
  let injectAnomalies = $state(true);

  async function startStream() {
    try {
      error = null;
      const response = await invoke<StreamResponse>('start_stream');
      if (response.success) {
        streamStatus = 'running';
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function stopStream() {
    try {
      error = null;
      const response = await invoke<StreamResponse>('stop_stream');
      if (response.success) {
        streamStatus = 'idle';
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function pauseStream() {
    try {
      error = null;
      const response = await invoke<StreamResponse>('pause_stream');
      if (response.success) {
        streamStatus = 'paused';
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function resumeStream() {
    try {
      error = null;
      const response = await invoke<StreamResponse>('resume_stream');
      if (response.success) {
        streamStatus = 'running';
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function bulkGenerate() {
    try {
      error = null;
      generating = true;
      lastResult = null;

      const response = await invoke<BulkGenerateResponse>('bulk_generate', {
        request: {
          entry_count: entryCount,
          include_master_data: includeMasterData,
          inject_anomalies: injectAnomalies,
        },
      });

      lastResult = response;
    } catch (e) {
      error = String(e);
    } finally {
      generating = false;
    }
  }
</script>

<section class="control-panel">
  <header class="panel-header">
    <h2>Controls</h2>
    <div class="stream-status">
      <span class="status-dot" class:active={streamStatus === 'running'} class:warning={streamStatus === 'paused'} class:inactive={streamStatus === 'idle'}></span>
      <span class="status-text">{streamStatus === 'running' ? 'Streaming' : streamStatus === 'paused' ? 'Paused' : 'Idle'}</span>
    </div>
  </header>

  <!-- Stream Controls -->
  <div class="control-section">
    <h3>Stream Control</h3>
    <div class="button-group">
      {#if streamStatus === 'idle'}
        <button class="btn-success" onclick={startStream} disabled={!connected}>
          Start Stream
        </button>
      {:else if streamStatus === 'running'}
        <button class="btn-secondary" onclick={pauseStream} disabled={!connected}>
          Pause
        </button>
        <button class="btn-danger" onclick={stopStream} disabled={!connected}>
          Stop
        </button>
      {:else}
        <button class="btn-success" onclick={resumeStream} disabled={!connected}>
          Resume
        </button>
        <button class="btn-danger" onclick={stopStream} disabled={!connected}>
          Stop
        </button>
      {/if}
    </div>
  </div>

  <div class="divider"></div>

  <!-- Bulk Generation -->
  <div class="control-section">
    <h3>Bulk Generation</h3>

    <div class="form-group">
      <label for="entry-count">Entry Count</label>
      <input
        type="number"
        id="entry-count"
        bind:value={entryCount}
        min="1"
        max="1000000"
        disabled={!connected || generating}
      />
    </div>

    <div class="form-group checkbox-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={includeMasterData} disabled={!connected || generating} />
        <span>Include Master Data</span>
      </label>
    </div>

    <div class="form-group checkbox-group">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={injectAnomalies} disabled={!connected || generating} />
        <span>Inject Anomalies</span>
      </label>
    </div>

    <button class="btn-primary full-width" onclick={bulkGenerate} disabled={!connected || generating}>
      {generating ? 'Generating...' : 'Generate'}
    </button>

    {#if lastResult}
      <div class="result-card">
        <div class="result-row">
          <span>Entries Generated</span>
          <span class="mono">{lastResult.entries_generated.toLocaleString()}</span>
        </div>
        <div class="result-row">
          <span>Anomalies</span>
          <span class="mono">{lastResult.anomaly_count.toLocaleString()}</span>
        </div>
        <div class="result-row">
          <span>Duration</span>
          <span class="mono">{lastResult.duration_ms}ms</span>
        </div>
      </div>
    {/if}
  </div>

  {#if error}
    <div class="error-message">
      <p>{error}</p>
    </div>
  {/if}
</section>

<style>
  .control-panel {
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

  .stream-status {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .status-text {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .control-section {
    margin-bottom: var(--space-4);
  }

  .control-section h3 {
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: var(--space-3);
  }

  .button-group {
    display: flex;
    gap: var(--space-2);
  }

  .button-group button {
    flex: 1;
  }

  .form-group {
    margin-bottom: var(--space-3);
  }

  .form-group input[type="number"] {
    width: 100%;
  }

  .checkbox-group {
    display: flex;
    align-items: center;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    text-transform: none;
    letter-spacing: normal;
  }

  .checkbox-label input {
    width: 16px;
    height: 16px;
  }

  .full-width {
    width: 100%;
  }

  .result-card {
    margin-top: var(--space-3);
    padding: var(--space-3);
    background-color: var(--color-background);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }

  .result-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-1) 0;
    font-size: 0.8125rem;
  }

  .result-row:not(:last-child) {
    border-bottom: 1px solid var(--color-border);
  }

  .result-row span:first-child {
    color: var(--color-text-muted);
  }

  .error-message {
    margin-top: var(--space-3);
    padding: var(--space-3);
    background-color: rgba(220, 53, 69, 0.1);
    border: 1px solid var(--color-danger);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    color: var(--color-danger);
  }
</style>
