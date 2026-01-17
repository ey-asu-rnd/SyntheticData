<script lang="ts">
  import { configStore, INDUSTRIES, COA_COMPLEXITIES } from '$lib/stores/config';

  const config = configStore.config;
  const isDirty = configStore.isDirty;
  const isValid = configStore.isValid;
  const validationErrors = configStore.validationErrors;
  const saving = configStore.saving;

  const sections = [
    {
      title: 'Global Settings',
      description: 'Industry, time period, and performance',
      href: '/config/global',
      icon: 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10',
    },
    {
      title: 'Companies',
      description: 'Company codes, currencies, and volumes',
      href: '/config/companies',
      icon: 'M6 22V4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v18M6 22h12M10 6h.01M14 6h.01',
    },
    {
      title: 'Transactions',
      description: 'Line items, amounts, and seasonality',
      href: '/config/transactions',
      icon: 'M18 20V10M12 20V4M6 20v-6',
    },
    {
      title: 'Master Data',
      description: 'Vendors, customers, materials, assets',
      href: '/config/master-data',
      icon: 'M21 5c0 1.1-4 2-9 2s-9-.9-9-2m18 0c0-1.1-4-2-9-2s-9 .9-9 2m18 0v14c0 1.1-4 2-9 2s-9-.9-9-2V5',
    },
    {
      title: 'Document Flows',
      description: 'P2P, O2C process chains',
      href: '/config/document-flows',
      icon: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5',
    },
    {
      title: 'Fraud & Controls',
      description: 'Anomaly injection and SOX controls',
      href: '/config/compliance',
      icon: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10zM9 12l2 2 4-4',
    },
    {
      title: 'Output',
      description: 'File formats and compression',
      href: '/config/output',
      icon: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12',
    },
  ];

  function getIndustryLabel(value: string): string {
    return INDUSTRIES.find(i => i.value === value)?.label ?? value;
  }

  function getComplexityLabel(value: string): string {
    return COA_COMPLEXITIES.find(c => c.value === value)?.label ?? value;
  }

  async function handleSave() {
    await configStore.save();
  }
</script>

<div class="page">
  <header class="page-header">
    <div>
      <h1>Configuration Overview</h1>
      <p>Manage all synthetic data generation settings</p>
    </div>
    <div class="header-actions">
      {#if $isDirty}
        <button class="btn-secondary" onclick={() => configStore.reset()}>
          Discard
        </button>
      {/if}
      <button
        class="btn-primary"
        onclick={handleSave}
        disabled={$saving || !$isDirty}
      >
        {$saving ? 'Saving...' : 'Save All'}
      </button>
    </div>
  </header>

  {#if $validationErrors.length > 0}
    <div class="validation-banner">
      <div class="banner-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
      </div>
      <div class="banner-content">
        <strong>Configuration has validation errors</strong>
        <ul>
          {#each $validationErrors.slice(0, 3) as error}
            <li>{error.message}</li>
          {/each}
          {#if $validationErrors.length > 3}
            <li>...and {$validationErrors.length - 3} more</li>
          {/if}
        </ul>
      </div>
    </div>
  {/if}

  {#if $config}
    <div class="summary-grid">
      <div class="summary-card">
        <div class="summary-label">Industry</div>
        <div class="summary-value">{getIndustryLabel($config.global.industry)}</div>
      </div>
      <div class="summary-card">
        <div class="summary-label">Period</div>
        <div class="summary-value">{$config.global.period_months} months</div>
      </div>
      <div class="summary-card">
        <div class="summary-label">Companies</div>
        <div class="summary-value">{$config.companies.length}</div>
      </div>
      <div class="summary-card">
        <div class="summary-label">CoA Complexity</div>
        <div class="summary-value">{getComplexityLabel($config.chart_of_accounts.complexity)}</div>
      </div>
    </div>
  {/if}

  <div class="sections-grid">
    {#each sections as section}
      <a href={section.href} class="section-card">
        <div class="section-icon">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d={section.icon} />
          </svg>
        </div>
        <div class="section-content">
          <h3 class="section-title">{section.title}</h3>
          <p class="section-description">{section.description}</p>
        </div>
        <div class="section-arrow">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18l6-6-6-6" />
          </svg>
        </div>
      </a>
    {/each}
  </div>

  <div class="status-section">
    <h2>Configuration Status</h2>
    <div class="status-grid">
      <div class="status-item" class:success={$isValid}>
        <span class="status-dot" class:active={$isValid} class:error={!$isValid}></span>
        <span>Validation: {$isValid ? 'Valid' : 'Has errors'}</span>
      </div>
      <div class="status-item" class:warning={$isDirty}>
        <span class="status-dot" class:warning={$isDirty} class:inactive={!$isDirty}></span>
        <span>Changes: {$isDirty ? 'Unsaved' : 'Saved'}</span>
      </div>
    </div>
  </div>
</div>

<style>
  .page {
    max-width: 1000px;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-6);
  }

  .page-header h1 {
    margin-bottom: var(--space-1);
  }

  .header-actions {
    display: flex;
    gap: var(--space-2);
  }

  .validation-banner {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-4);
    background-color: rgba(255, 193, 7, 0.1);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-lg);
    margin-bottom: var(--space-6);
  }

  .banner-icon {
    width: 24px;
    height: 24px;
    color: var(--color-warning);
    flex-shrink: 0;
  }

  .banner-icon svg {
    width: 100%;
    height: 100%;
  }

  .banner-content {
    font-size: 0.875rem;
  }

  .banner-content strong {
    display: block;
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .banner-content ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    color: var(--color-text-secondary);
    font-size: 0.8125rem;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: var(--space-4);
    margin-bottom: var(--space-6);
  }

  .summary-card {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .summary-label {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--space-1);
  }

  .summary-value {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .sections-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-4);
    margin-bottom: var(--space-6);
  }

  .section-card {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-4);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    text-decoration: none;
    color: inherit;
    transition: all var(--transition-fast);
  }

  .section-card:hover {
    border-color: var(--color-accent);
    box-shadow: var(--shadow-md);
  }

  .section-icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    color: var(--color-accent);
    flex-shrink: 0;
  }

  .section-icon svg {
    width: 20px;
    height: 20px;
  }

  .section-content {
    flex: 1;
    min-width: 0;
  }

  .section-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .section-description {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .section-arrow {
    width: 20px;
    height: 20px;
    color: var(--color-text-muted);
    transition: transform var(--transition-fast);
  }

  .section-card:hover .section-arrow {
    transform: translateX(4px);
    color: var(--color-accent);
  }

  .section-arrow svg {
    width: 100%;
    height: 100%;
  }

  .status-section {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .status-section h2 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--space-3);
  }

  .status-grid {
    display: flex;
    gap: var(--space-5);
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 0.875rem;
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .summary-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .sections-grid {
      grid-template-columns: 1fr;
    }

    .page-header {
      flex-direction: column;
      gap: var(--space-4);
    }
  }
</style>
