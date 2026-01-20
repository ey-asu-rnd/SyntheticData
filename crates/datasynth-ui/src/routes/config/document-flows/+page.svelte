<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { Toggle } from '$lib/components/forms';

  const config = configStore.config;

  const flows = [
    {
      key: 'p2p' as const,
      title: 'Procure to Pay (P2P)',
      description: 'Purchase orders, goods receipts, vendor invoices, and payments',
      icon: 'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v0a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2zM9 12h6M9 16h6',
      href: '/config/document-flows/p2p',
      steps: ['Purchase Order', 'Goods Receipt', 'Invoice Receipt', 'Payment'],
    },
    {
      key: 'o2c' as const,
      title: 'Order to Cash (O2C)',
      description: 'Sales orders, deliveries, customer invoices, and receipts',
      icon: 'M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1zM12 11v6M9 14l3 3 3-3',
      href: '/config/document-flows/o2c',
      steps: ['Sales Order', 'Delivery', 'Customer Invoice', 'Receipt'],
    },
  ];

  function getFlowEnabled(key: 'p2p' | 'o2c'): boolean {
    if (!$config?.document_flows) return false;
    return $config.document_flows[key]?.enabled ?? false;
  }

  function getFlowRate(key: 'p2p' | 'o2c'): number {
    // For P2P, use three_way_match_rate as a proxy for activity rate
    // For O2C, return a simple 1.0 since it doesn't have a rate field
    if (!$config?.document_flows) return 0;
    const flow = $config.document_flows[key];
    if (!flow?.enabled) return 0;
    // Return 1.0 if enabled, representing "active"
    return 1.0;
  }

  function toggleFlow(key: 'p2p' | 'o2c') {
    if (!$config?.document_flows) return;
    const flow = $config.document_flows[key];
    if (flow) {
      flow.enabled = !flow.enabled;
      configStore.set($config);
    }
  }
</script>

<div class="page">
  <header class="page-header">
    <div>
      <h1>Document Flows</h1>
      <p>Configure end-to-end business process document chains</p>
    </div>
  </header>

  <div class="flows-grid">
    {#each flows as flow}
      {@const flowEnabled = getFlowEnabled(flow.key)}
      {@const flowRate = getFlowRate(flow.key)}
      <div class="flow-card" class:enabled={flowEnabled}>
        <div class="flow-header">
          <div class="flow-icon">
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d={flow.icon} />
            </svg>
          </div>
          <div class="flow-title-group">
            <h3 class="flow-title">{flow.title}</h3>
            <p class="flow-description">{flow.description}</p>
          </div>
          <div class="flow-toggle">
            <button
              class="toggle-btn"
              class:active={flowEnabled}
              onclick={() => toggleFlow(flow.key)}
              aria-label={flowEnabled ? `Disable ${flow.title}` : `Enable ${flow.title}`}
            >
              <span class="toggle-track">
                <span class="toggle-thumb"></span>
              </span>
            </button>
          </div>
        </div>

        <div class="flow-steps">
          {#each flow.steps as step, i}
            <div class="step" class:dimmed={!flowEnabled}>
              <span class="step-number">{i + 1}</span>
              <span class="step-name">{step}</span>
            </div>
            {#if i < flow.steps.length - 1}
              <div class="step-arrow" class:dimmed={!flowEnabled}>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M5 12h14M12 5l7 7-7 7" />
                </svg>
              </div>
            {/if}
          {/each}
        </div>

        <div class="flow-footer">
          <div class="flow-stat">
            <span class="stat-value">{flowEnabled ? 'Enabled' : 'Disabled'}</span>
            <span class="stat-label">{flowEnabled ? 'generating documents' : 'not active'}</span>
          </div>
          <a href={flow.href} class="flow-link">
            Configure
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 18l6-6-6-6" />
            </svg>
          </a>
        </div>
      </div>
    {/each}
  </div>

  <div class="info-section">
    <h2>About Document Flows</h2>
    <p>
      Document flows represent complete business processes that span multiple
      documents and transactions. Each flow generates properly linked documents
      with realistic timing, quantities, and amounts.
    </p>

    <div class="flow-details">
      <div class="flow-detail">
        <h4>Procure to Pay (P2P)</h4>
        <ul>
          <li>Purchase Order creation with line items</li>
          <li>Goods Receipt with inventory posting</li>
          <li>Three-way matching (PO/GR/Invoice)</li>
          <li>Invoice verification and AP posting</li>
          <li>Payment execution with clearing</li>
        </ul>
      </div>

      <div class="flow-detail">
        <h4>Order to Cash (O2C)</h4>
        <ul>
          <li>Sales Order with pricing and availability</li>
          <li>Delivery with picking and goods issue</li>
          <li>Customer Invoice with revenue recognition</li>
          <li>Cash receipt and AR clearing</li>
          <li>Dunning for late payments</li>
        </ul>
      </div>
    </div>
  </div>
</div>

<style>
  .page {
    max-width: 1000px;
  }

  .page-header {
    margin-bottom: var(--space-6);
  }

  .page-header h1 {
    margin-bottom: var(--space-1);
  }

  .flows-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin-bottom: var(--space-8);
  }

  .flow-card {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
    transition: border-color var(--transition-fast);
  }

  .flow-card.enabled {
    border-color: var(--color-accent);
  }

  .flow-header {
    display: flex;
    align-items: flex-start;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  .flow-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    color: var(--color-accent);
    flex-shrink: 0;
  }

  .flow-icon svg {
    width: 24px;
    height: 24px;
  }

  .flow-title-group {
    flex: 1;
  }

  .flow-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .flow-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .flow-steps {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-4);
    flex-wrap: wrap;
  }

  .step {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background-color: var(--color-surface);
    border-radius: var(--radius-md);
    transition: opacity var(--transition-fast);
  }

  .step.dimmed {
    opacity: 0.5;
  }

  .step-number {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.6875rem;
    font-weight: 600;
    color: white;
    background-color: var(--color-accent);
    border-radius: 50%;
  }

  .step-name {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .step-arrow {
    width: 24px;
    height: 24px;
    color: var(--color-text-muted);
    transition: opacity var(--transition-fast);
  }

  .step-arrow.dimmed {
    opacity: 0.3;
  }

  .step-arrow svg {
    width: 100%;
    height: 100%;
  }

  .flow-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .flow-stat {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }

  .stat-value {
    font-family: var(--font-mono);
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .stat-label {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
  }

  .flow-link {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-accent);
    text-decoration: none;
  }

  .flow-link svg {
    width: 16px;
    height: 16px;
    transition: transform var(--transition-fast);
  }

  .flow-link:hover svg {
    transform: translateX(4px);
  }

  .info-section {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
  }

  .info-section h2 {
    font-size: 0.9375rem;
    font-weight: 600;
    margin-bottom: var(--space-3);
  }

  .info-section > p {
    font-size: 0.875rem;
    margin-bottom: var(--space-5);
  }

  .flow-details {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-5);
  }

  .flow-detail h4 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .flow-detail ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .flow-detail li {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    padding-left: var(--space-3);
    position: relative;
  }

  .flow-detail li::before {
    content: '';
    position: absolute;
    left: 0;
    top: 0.5em;
    width: 4px;
    height: 4px;
    background-color: var(--color-accent);
    border-radius: 50%;
  }

  .toggle-btn {
    position: relative;
    width: 44px;
    height: 24px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }

  .toggle-btn .toggle-track {
    display: block;
    width: 100%;
    height: 100%;
    background-color: var(--color-border);
    border-radius: 12px;
    transition: background-color var(--transition-fast);
  }

  .toggle-btn.active .toggle-track {
    background-color: var(--color-accent);
  }

  .toggle-btn .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background-color: white;
    border-radius: 50%;
    transition: transform var(--transition-fast);
    box-shadow: var(--shadow-sm);
  }

  .toggle-btn.active .toggle-thumb {
    transform: translateX(20px);
  }

  @media (max-width: 768px) {
    .flow-details {
      grid-template-columns: 1fr;
    }

    .flow-steps {
      flex-direction: column;
    }

    .step-arrow {
      transform: rotate(90deg);
    }
  }
</style>
