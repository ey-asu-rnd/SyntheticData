<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  interface StreamEvent {
    sequence: number;
    timestamp: string;
    event_type: string;
    document_id: string;
    company_code: string;
    amount: string;
    is_anomaly: boolean;
  }

  let events: StreamEvent[] = $state([]);
  let connected = $state(false);
  let socket: WebSocket | null = null;
  let serverUrl = $state('ws://localhost:3000/ws/events');
  let autoScroll = $state(true);
  let maxEvents = 100;

  function connect() {
    if (socket) {
      socket.close();
    }

    socket = new WebSocket(serverUrl);

    socket.onopen = () => {
      connected = true;
      events = [];
    };

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as StreamEvent;
        events = [...events.slice(-maxEvents + 1), data];

        if (autoScroll) {
          requestAnimationFrame(() => {
            const container = document.querySelector('.events-list');
            if (container) {
              container.scrollTop = container.scrollHeight;
            }
          });
        }
      } catch (e) {
        console.error('Failed to parse event:', e);
      }
    };

    socket.onclose = () => {
      connected = false;
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      connected = false;
    };
  }

  function disconnect() {
    if (socket) {
      socket.close();
      socket = null;
    }
    connected = false;
  }

  function clearEvents() {
    events = [];
  }

  function formatTimestamp(ts: string): string {
    const date = new Date(ts);
    return date.toLocaleTimeString('en-US', { hour12: false, fractionalSecondDigits: 3 });
  }

  function formatAmount(amount: string): string {
    const num = parseFloat(amount);
    return num.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  onDestroy(() => {
    if (socket) {
      socket.close();
    }
  });
</script>

<div class="stream-page">
  <div class="page-header">
    <div>
      <h1>Event Stream</h1>
      <p>Real-time view of generated events via WebSocket</p>
    </div>
    <div class="header-actions">
      <div class="connection-status">
        <span class="status-dot" class:active={connected} class:inactive={!connected}></span>
        <span>{connected ? 'Connected' : 'Disconnected'}</span>
      </div>
    </div>
  </div>

  <!-- Connection Panel -->
  <section class="connection-panel">
    <div class="connection-form">
      <div class="form-group">
        <label for="server-url">WebSocket URL</label>
        <input
          type="text"
          id="server-url"
          bind:value={serverUrl}
          placeholder="ws://localhost:3000/ws/events"
          disabled={connected}
        />
      </div>
      <div class="form-actions">
        {#if connected}
          <button class="btn-danger" onclick={disconnect}>Disconnect</button>
        {:else}
          <button class="btn-success" onclick={connect}>Connect</button>
        {/if}
        <button class="btn-secondary" onclick={clearEvents} disabled={events.length === 0}>
          Clear Events
        </button>
      </div>
    </div>

    <div class="stream-options">
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={autoScroll} />
        <span>Auto-scroll</span>
      </label>
      <span class="event-count mono">{events.length} events</span>
    </div>
  </section>

  <!-- Events Table -->
  <section class="events-section">
    <div class="events-header">
      <div class="col-seq">#</div>
      <div class="col-time">Time</div>
      <div class="col-type">Type</div>
      <div class="col-doc">Document ID</div>
      <div class="col-company">Company</div>
      <div class="col-amount">Amount</div>
      <div class="col-status">Status</div>
    </div>

    <div class="events-list">
      {#if events.length === 0}
        <div class="no-events">
          <p>{connected ? 'Waiting for events...' : 'Connect to start receiving events'}</p>
        </div>
      {:else}
        {#each events as event}
          <div class="event-row" class:anomaly={event.is_anomaly}>
            <div class="col-seq mono">{event.sequence}</div>
            <div class="col-time mono">{formatTimestamp(event.timestamp)}</div>
            <div class="col-type">{event.event_type}</div>
            <div class="col-doc mono">{event.document_id.slice(0, 8)}...</div>
            <div class="col-company mono">{event.company_code}</div>
            <div class="col-amount mono">{formatAmount(event.amount)}</div>
            <div class="col-status">
              {#if event.is_anomaly}
                <span class="badge danger">Anomaly</span>
              {:else}
                <span class="badge">Normal</span>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </section>
</div>

<style>
  .stream-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    height: calc(100vh - 200px);
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .page-header h1 {
    margin-bottom: var(--space-1);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .connection-status {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.875rem;
    font-weight: 500;
  }

  .connection-panel {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
  }

  .connection-form {
    display: flex;
    gap: var(--space-4);
    align-items: flex-end;
  }

  .connection-form .form-group {
    min-width: 400px;
  }

  .form-actions {
    display: flex;
    gap: var(--space-2);
  }

  .stream-options {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
  }

  .checkbox-label input {
    width: 16px;
    height: 16px;
  }

  .event-count {
    font-size: 0.8125rem;
    color: var(--color-text-muted);
  }

  .events-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .events-header {
    display: grid;
    grid-template-columns: 60px 100px 120px 1fr 80px 120px 100px;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background-color: var(--color-background);
    border-bottom: 1px solid var(--color-border);
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .events-list {
    flex: 1;
    overflow-y: auto;
  }

  .event-row {
    display: grid;
    grid-template-columns: 60px 100px 120px 1fr 80px 120px 100px;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    font-size: 0.8125rem;
    border-bottom: 1px solid var(--color-border);
    align-items: center;
  }

  .event-row:hover {
    background-color: var(--color-background);
  }

  .event-row.anomaly {
    background-color: rgba(220, 53, 69, 0.03);
  }

  .col-seq {
    color: var(--color-text-muted);
  }

  .col-time {
    color: var(--color-text-secondary);
  }

  .col-type {
    font-weight: 500;
  }

  .col-doc {
    color: var(--color-text-secondary);
  }

  .col-company {
    font-weight: 500;
    color: var(--color-accent);
  }

  .col-amount {
    text-align: right;
    font-weight: 500;
  }

  .no-events {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-muted);
  }

  @media (max-width: 1024px) {
    .events-header,
    .event-row {
      grid-template-columns: 50px 80px 100px 1fr 60px 100px 80px;
      font-size: 0.75rem;
    }

    .connection-form .form-group {
      min-width: 250px;
    }
  }
</style>
