<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface CompanyConfigDto {
    code: string;
    name: string;
    currency: string;
    country: string;
    annual_transaction_volume: number;
    volume_weight: number;
  }

  interface GenerationConfigDto {
    industry: string;
    start_date: string;
    period_months: number;
    seed: number | null;
    coa_complexity: string;
    companies: CompanyConfigDto[];
    fraud_enabled: boolean;
    fraud_rate: number;
  }

  interface ConfigResponse {
    success: boolean;
    message: string;
    config: GenerationConfigDto | null;
  }

  let config: GenerationConfigDto | null = $state(null);
  let loading = $state(true);
  let saving = $state(false);
  let error: string | null = $state(null);
  let success: string | null = $state(null);

  const industries = ['Manufacturing', 'Retail', 'FinancialServices', 'Healthcare', 'Technology'];
  const complexities = ['Small', 'Medium', 'Large'];

  async function loadConfig() {
    try {
      loading = true;
      error = null;
      const response = await invoke<ConfigResponse>('get_config');
      if (response.success && response.config) {
        config = response.config;
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function saveConfig() {
    if (!config) return;
    try {
      saving = true;
      error = null;
      success = null;
      const response = await invoke<ConfigResponse>('set_config', { config });
      if (response.success) {
        success = 'Configuration saved successfully';
        setTimeout(() => success = null, 3000);
      }
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function addCompany() {
    if (!config) return;
    config.companies = [...config.companies, {
      code: `${1000 + config.companies.length}`,
      name: 'New Company',
      currency: 'USD',
      country: 'US',
      annual_transaction_volume: 100000,
      volume_weight: 1.0,
    }];
  }

  function removeCompany(index: number) {
    if (!config || config.companies.length <= 1) return;
    config.companies = config.companies.filter((_, i) => i !== index);
  }

  onMount(loadConfig);
</script>

<div class="config-page">
  <div class="page-header">
    <div>
      <h1>Configuration</h1>
      <p>Configure generation parameters and company settings</p>
    </div>
    <button class="btn-primary" onclick={saveConfig} disabled={loading || saving || !config}>
      {saving ? 'Saving...' : 'Save Configuration'}
    </button>
  </div>

  {#if error}
    <div class="alert alert-error">
      <p>{error}</p>
    </div>
  {/if}

  {#if success}
    <div class="alert alert-success">
      <p>{success}</p>
    </div>
  {/if}

  {#if loading}
    <div class="loading">
      <p>Loading configuration...</p>
    </div>
  {:else if config}
    <div class="config-grid">
      <!-- Global Settings -->
      <section class="config-section">
        <h2>Global Settings</h2>
        <div class="form-grid">
          <div class="form-group">
            <label for="industry">Industry</label>
            <select id="industry" bind:value={config.industry}>
              {#each industries as industry}
                <option value={industry}>{industry}</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label for="complexity">CoA Complexity</label>
            <select id="complexity" bind:value={config.coa_complexity}>
              {#each complexities as complexity}
                <option value={complexity}>{complexity}</option>
              {/each}
            </select>
          </div>

          <div class="form-group">
            <label for="start-date">Start Date</label>
            <input type="text" id="start-date" bind:value={config.start_date} placeholder="YYYY-MM-DD" />
          </div>

          <div class="form-group">
            <label for="period-months">Period (Months)</label>
            <input type="number" id="period-months" bind:value={config.period_months} min="1" max="60" />
          </div>

          <div class="form-group">
            <label for="seed">Random Seed</label>
            <input type="number" id="seed" bind:value={config.seed} placeholder="Optional" />
          </div>
        </div>
      </section>

      <!-- Fraud Settings -->
      <section class="config-section">
        <h2>Fraud / Anomaly Settings</h2>
        <div class="form-grid">
          <div class="form-group checkbox-inline">
            <label class="checkbox-label">
              <input type="checkbox" bind:checked={config.fraud_enabled} />
              <span>Enable Fraud Injection</span>
            </label>
          </div>

          <div class="form-group">
            <label for="fraud-rate">Fraud Rate (%)</label>
            <input
              type="number"
              id="fraud-rate"
              bind:value={config.fraud_rate}
              min="0"
              max="100"
              step="0.1"
              disabled={!config.fraud_enabled}
            />
          </div>
        </div>
      </section>

      <!-- Companies -->
      <section class="config-section full-width">
        <div class="section-header">
          <h2>Companies</h2>
          <button class="btn-secondary" onclick={addCompany}>Add Company</button>
        </div>

        <div class="companies-list">
          {#each config.companies as company, index}
            <div class="company-card">
              <div class="company-header">
                <span class="company-code mono">{company.code}</span>
                {#if config.companies.length > 1}
                  <button class="btn-icon" onclick={() => removeCompany(index)} title="Remove company">
                    &times;
                  </button>
                {/if}
              </div>

              <div class="company-form">
                <div class="form-group">
                  <label>Company Code</label>
                  <input type="text" bind:value={company.code} />
                </div>

                <div class="form-group">
                  <label>Name</label>
                  <input type="text" bind:value={company.name} />
                </div>

                <div class="form-group">
                  <label>Currency</label>
                  <input type="text" bind:value={company.currency} maxlength="3" />
                </div>

                <div class="form-group">
                  <label>Country</label>
                  <input type="text" bind:value={company.country} maxlength="2" />
                </div>

                <div class="form-group">
                  <label>Annual Volume</label>
                  <input type="number" bind:value={company.annual_transaction_volume} min="1000" />
                </div>

                <div class="form-group">
                  <label>Volume Weight</label>
                  <input type="number" bind:value={company.volume_weight} min="0" max="10" step="0.1" />
                </div>
              </div>
            </div>
          {/each}
        </div>
      </section>
    </div>
  {/if}
</div>

<style>
  .config-page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .page-header h1 {
    margin-bottom: var(--space-1);
  }

  .config-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-5);
  }

  .config-section {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
  }

  .config-section.full-width {
    grid-column: 1 / -1;
  }

  .config-section h2 {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: var(--space-4);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-4);
  }

  .section-header h2 {
    margin-bottom: 0;
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .form-group input,
  .form-group select {
    width: 100%;
  }

  .checkbox-inline {
    flex-direction: row;
    align-items: center;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    cursor: pointer;
    font-size: 0.875rem;
    color: var(--color-text-primary);
  }

  .checkbox-label input {
    width: 16px;
    height: 16px;
  }

  .companies-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: var(--space-4);
  }

  .company-card {
    background-color: var(--color-background);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--space-4);
  }

  .company-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--color-border);
  }

  .company-code {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-accent);
  }

  .btn-icon {
    width: 24px;
    height: 24px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.25rem;
    color: var(--color-text-muted);
    background: none;
    border: none;
  }

  .btn-icon:hover {
    color: var(--color-danger);
  }

  .company-form {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-3);
  }

  .company-form .form-group label {
    font-size: 0.6875rem;
  }

  .company-form .form-group input {
    padding: var(--space-1) var(--space-2);
    font-size: 0.8125rem;
  }

  .alert {
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
  }

  .alert-error {
    background-color: rgba(220, 53, 69, 0.1);
    border: 1px solid var(--color-danger);
    color: var(--color-danger);
  }

  .alert-success {
    background-color: rgba(40, 167, 69, 0.1);
    border: 1px solid var(--color-success);
    color: var(--color-success);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 1024px) {
    .config-grid {
      grid-template-columns: 1fr;
    }

    .form-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
