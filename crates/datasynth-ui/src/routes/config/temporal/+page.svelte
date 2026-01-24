<script lang="ts">
  import { configStore, DRIFT_TYPES } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';

  const config = configStore.config;
  const isDirty = configStore.isDirty;
  const saving = configStore.saving;
  const validationErrors = configStore.validationErrors;

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  async function handleSave() {
    await configStore.save();
  }
</script>

<div class="page">
  <header class="page-header">
    <div>
      <h1>Temporal Drift</h1>
      <p>Simulate distribution changes over time for drift detection training</p>
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
        {$saving ? 'Saving...' : 'Save Changes'}
      </button>
    </div>
  </header>

  {#if $config}
    <div class="page-content">
      <FormSection title="Drift Simulation" description="Enable temporal drift to simulate realistic data evolution">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.temporal.enabled}
              label="Enable Temporal Drift"
              description="Generate data that shows realistic temporal evolution"
            />

            {#if $config.temporal.enabled}
              <FormGroup
                label="Drift Type"
                htmlFor="drift-type"
                helpText="Select the type of drift pattern to simulate"
              >
                {#snippet children()}
                  <div class="drift-type-selector">
                    {#each DRIFT_TYPES as type}
                      <label class="drift-type-option" class:selected={$config.temporal.drift_type === type.value}>
                        <input
                          type="radio"
                          name="drift-type"
                          value={type.value}
                          bind:group={$config.temporal.drift_type}
                        />
                        <span class="drift-type-label">{type.label}</span>
                        <span class="drift-type-desc">{type.description}</span>
                      </label>
                    {/each}
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Drift Start Period"
                htmlFor="drift-start"
                helpText="Period (month) when drift begins (0 = from start)"
              >
                {#snippet children()}
                  <input
                    type="number"
                    id="drift-start"
                    bind:value={$config.temporal.drift_start_period}
                    min="0"
                    max="120"
                  />
                {/snippet}
              </FormGroup>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      {#if $config.temporal.enabled}
        <FormSection title="Amount Distribution Drift" description="How transaction amounts change over time">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Mean Drift"
                htmlFor="amount-mean-drift"
                helpText="Amount mean shift per period (e.g., 0.02 = 2% increase per month)"
                error={getError('temporal.amount_mean_drift')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="amount-mean-drift"
                      bind:value={$config.temporal.amount_mean_drift}
                      min="-0.1"
                      max="0.1"
                      step="0.005"
                    />
                    <span class="slider-value">{($config.temporal.amount_mean_drift * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Variance Drift"
                htmlFor="amount-variance-drift"
                helpText="Amount variance increase per period (simulates increasing volatility)"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="amount-variance-drift"
                      bind:value={$config.temporal.amount_variance_drift}
                      min="0"
                      max="0.1"
                      step="0.005"
                    />
                    <span class="slider-value">{($config.temporal.amount_variance_drift * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Anomaly & Concept Drift" description="How patterns and anomaly rates evolve">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Anomaly Rate Drift"
                htmlFor="anomaly-drift"
                helpText="Increase in anomaly rate per period (simulates degrading controls)"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="anomaly-drift"
                      bind:value={$config.temporal.anomaly_rate_drift}
                      min="0"
                      max="0.01"
                      step="0.0005"
                    />
                    <span class="slider-value">{($config.temporal.anomaly_rate_drift * 100).toFixed(2)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Concept Drift Rate"
                htmlFor="concept-drift"
                helpText="Rate of feature distribution changes (0-1, higher = faster changes)"
                error={getError('temporal.concept_drift_rate')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="concept-drift"
                      bind:value={$config.temporal.concept_drift_rate}
                      min="0"
                      max="0.1"
                      step="0.005"
                    />
                    <span class="slider-value">{($config.temporal.concept_drift_rate * 100).toFixed(1)}%</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>
          {/snippet}
        </FormSection>

        <FormSection title="Sudden Drift Events" description="Configure occasional sudden shifts in distributions">
          {#snippet children()}
            <div class="form-grid">
              <FormGroup
                label="Sudden Drift Probability"
                htmlFor="sudden-prob"
                helpText="Probability of a sudden shift occurring in any period"
                error={getError('temporal.sudden_drift_probability')}
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="sudden-prob"
                      bind:value={$config.temporal.sudden_drift_probability}
                      min="0"
                      max="0.2"
                      step="0.01"
                    />
                    <span class="slider-value">{($config.temporal.sudden_drift_probability * 100).toFixed(0)}%</span>
                  </div>
                {/snippet}
              </FormGroup>

              <FormGroup
                label="Sudden Drift Magnitude"
                htmlFor="sudden-mag"
                helpText="Magnitude multiplier when sudden drift occurs"
              >
                {#snippet children()}
                  <div class="slider-with-value">
                    <input
                      type="range"
                      id="sudden-mag"
                      bind:value={$config.temporal.sudden_drift_magnitude}
                      min="1"
                      max="5"
                      step="0.1"
                    />
                    <span class="slider-value">{$config.temporal.sudden_drift_magnitude.toFixed(1)}x</span>
                  </div>
                {/snippet}
              </FormGroup>
            </div>

            <Toggle
              bind:checked={$config.temporal.seasonal_drift}
              label="Enable Seasonal Drift"
              description="Add cyclic patterns that repeat annually"
            />
          {/snippet}
        </FormSection>
      {/if}

      <div class="info-section">
        <h2>About Temporal Drift</h2>
        <div class="info-grid">
          <div class="info-card">
            <h3>Use Cases</h3>
            <p>
              Temporal drift simulation is useful for training drift detection models,
              testing temporal robustness, and simulating realistic data evolution
              like inflation or changing fraud patterns.
            </p>
          </div>
          <div class="info-card">
            <h3>Drift Types</h3>
            <p>
              <strong>Gradual:</strong> Continuous drift like inflation.<br/>
              <strong>Sudden:</strong> Point-in-time shifts like policy changes.<br/>
              <strong>Recurring:</strong> Cyclic patterns like seasonal variations.
            </p>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div class="loading">
      <p>Loading configuration...</p>
    </div>
  {/if}
</div>

<style>
  .page {
    max-width: 900px;
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

  .page-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .form-stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .drift-type-selector {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-2);
  }

  .drift-type-option {
    display: flex;
    flex-direction: column;
    padding: var(--space-3);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .drift-type-option:hover {
    border-color: var(--color-accent);
  }

  .drift-type-option.selected {
    border-color: var(--color-accent);
    background-color: rgba(59, 130, 246, 0.05);
  }

  .drift-type-option input {
    display: none;
  }

  .drift-type-label {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .drift-type-desc {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .slider-with-value {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .slider-with-value input[type="range"] {
    flex: 1;
    height: 6px;
    border-radius: 3px;
    background: var(--color-background);
    appearance: none;
    cursor: pointer;
  }

  .slider-with-value input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
    border: 2px solid var(--color-surface);
    box-shadow: var(--shadow-sm);
  }

  .slider-value {
    min-width: 60px;
    text-align: right;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--color-text-primary);
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
    margin-bottom: var(--space-4);
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-4);
  }

  .info-card {
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
  }

  .info-card h3 {
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .info-card p {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    line-height: 1.5;
    margin: 0;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .form-grid,
    .drift-type-selector,
    .info-grid {
      grid-template-columns: 1fr;
    }

    .page-header {
      flex-direction: column;
      gap: var(--space-4);
    }
  }
</style>
