<script lang="ts">
  /**
   * Visual distribution editor with bars and drag adjustment.
   * Used for editing weighted distributions like line items, sources, etc.
   */

  // Use a generic type that accepts any object with string keys and number values
  type Distribution = { [key: string]: number };

  let {
    label = '',
    distribution = $bindable<Distribution>({}),
    labels = {} as Record<string, string>,
    descriptions = {} as Record<string, string>,
    helpText = '',
  }: {
    label?: string;
    distribution: Distribution;
    labels?: Record<string, string>;
    descriptions?: Record<string, string>;
    helpText?: string;
  } = $props();

  // Calculate total for normalization display
  let total = $derived(Object.values(distribution).reduce((sum, v) => sum + v, 0));

  // Get percentage for display
  function getPercent(value: number): number {
    return total > 0 ? (value / total) * 100 : 0;
  }

  // Update a value and normalize
  function updateValue(key: string, newPercent: number) {
    // Clamp to valid range
    newPercent = Math.max(0, Math.min(100, newPercent));

    // Calculate the new absolute value based on desired percentage
    const newValue = newPercent / 100;

    // Update the distribution
    distribution[key] = newValue;

    // Normalize all values to sum to 1
    const newTotal = Object.values(distribution).reduce((sum, v) => sum + v, 0);
    if (newTotal > 0) {
      for (const k of Object.keys(distribution)) {
        distribution[k] = distribution[k] / newTotal;
      }
    }
  }

  // Get display label for a key
  function getLabel(key: string): string {
    return labels[key] || key;
  }

  // Get description for a key
  function getDescription(key: string): string | undefined {
    return descriptions[key];
  }
</script>

<div class="distribution-editor">
  {#if label}
    <span class="editor-label">{label}</span>
  {/if}

  <div class="distribution-bars">
    {#each Object.entries(distribution) as [key, value]}
      {@const percent = getPercent(value)}
      {@const desc = getDescription(key)}
      <div class="bar-row" class:has-description={!!desc}>
        <div class="bar-label-container">
          <span class="bar-label">{getLabel(key)}</span>
          {#if desc}
            <span class="bar-description">{desc}</span>
          {/if}
        </div>
        <div class="bar-track">
          <div class="bar-fill" style="width: {percent}%"></div>
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            value={percent}
            oninput={(e) => updateValue(key, Number(e.currentTarget.value))}
            class="bar-slider"
          />
        </div>
        <span class="bar-value">{percent.toFixed(1)}%</span>
      </div>
    {/each}
  </div>

  {#if helpText}
    <p class="help-text">{helpText}</p>
  {/if}
</div>

<style>
  .distribution-editor {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .editor-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .distribution-bars {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .bar-row {
    display: grid;
    grid-template-columns: 160px 1fr 60px;
    align-items: center;
    gap: var(--space-3);
  }

  .bar-row.has-description {
    align-items: start;
  }

  .bar-label-container {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .bar-label {
    font-size: 0.8125rem;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .bar-description {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .bar-track {
    position: relative;
    height: 8px;
    background-color: var(--color-border);
    border-radius: 4px;
    overflow: visible;
  }

  .bar-fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background-color: var(--color-accent);
    border-radius: 4px;
    transition: width var(--transition-fast);
    pointer-events: none;
  }

  .bar-slider {
    position: absolute;
    top: 50%;
    left: 0;
    transform: translateY(-50%);
    width: 100%;
    height: 24px;
    opacity: 0;
    cursor: pointer;
    margin: 0;
  }

  .bar-track:hover .bar-fill {
    background-color: var(--color-accent-hover);
  }

  .bar-value {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    text-align: right;
  }

  .help-text {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin: 0;
    margin-top: var(--space-1);
  }
</style>
