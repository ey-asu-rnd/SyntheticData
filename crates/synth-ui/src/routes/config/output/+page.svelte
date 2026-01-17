<script lang="ts">
  import { configStore } from '$lib/stores/config';
  import { FormSection, FormGroup, Toggle } from '$lib/components/forms';

  const config = configStore.config;
  const isDirty = configStore.isDirty;
  const saving = configStore.saving;
  const validationErrors = configStore.validationErrors;

  const OUTPUT_MODES = [
    { value: 'streaming', label: 'Streaming', description: 'Stream records as generated' },
    { value: 'flat_file', label: 'Flat File', description: 'Write to files on disk' },
    { value: 'both', label: 'Both', description: 'Stream and write to files' },
  ];

  const FILE_FORMATS = [
    { value: 'parquet', label: 'Parquet', description: 'Columnar format, best for analytics' },
    { value: 'csv', label: 'CSV', description: 'Comma-separated values, widely compatible' },
    { value: 'json', label: 'JSON', description: 'Full JSON array' },
    { value: 'json_lines', label: 'JSON Lines', description: 'One JSON object per line' },
  ];

  const COMPRESSION_ALGORITHMS = [
    { value: 'zstd', label: 'Zstandard', description: 'Best balance of speed and compression' },
    { value: 'gzip', label: 'Gzip', description: 'Widely compatible' },
    { value: 'lz4', label: 'LZ4', description: 'Fastest compression' },
    { value: 'snappy', label: 'Snappy', description: 'Fast, moderate compression' },
  ];

  function getError(field: string): string {
    const found = $validationErrors.find(e => e.field === field);
    return found?.message || '';
  }

  function toggleFormat(format: string) {
    if (!$config?.output?.formats) return;
    const idx = $config.output.formats.indexOf(format);
    if (idx >= 0) {
      if ($config.output.formats.length > 1) {
        $config.output.formats = $config.output.formats.filter(f => f !== format);
      }
    } else {
      $config.output.formats = [...$config.output.formats, format];
    }
    configStore.set($config);
  }

  function isFormatSelected(format: string): boolean {
    return $config?.output?.formats?.includes(format) ?? false;
  }

  async function handleSave() {
    await configStore.save();
  }
</script>

<div class="page">
  <header class="page-header">
    <div>
      <h1>Output Settings</h1>
      <p>Configure file formats, compression, and partitioning</p>
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

  {#if $config?.output}
    <div class="page-content">
      <FormSection title="Output Mode" description="How generated data should be delivered">
        {#snippet children()}
          <div class="mode-selector">
            {#each OUTPUT_MODES as mode}
              <button
                class="mode-option"
                class:selected={$config.output.mode === mode.value}
                onclick={() => { $config.output.mode = mode.value; configStore.set($config); }}
              >
                <span class="mode-label">{mode.label}</span>
                <span class="mode-description">{mode.description}</span>
              </button>
            {/each}
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Output Directory" description="Where files will be written">
        {#snippet children()}
          <FormGroup
            label="Directory Path"
            htmlFor="output-dir"
            helpText="Relative or absolute path for output files"
            error={getError('output.output_directory')}
          >
            {#snippet children()}
              <input
                type="text"
                id="output-dir"
                bind:value={$config.output.output_directory}
                placeholder="./output"
              />
            {/snippet}
          </FormGroup>
        {/snippet}
      </FormSection>

      <FormSection title="File Formats" description="Select one or more output formats">
        {#snippet children()}
          <div class="format-grid">
            {#each FILE_FORMATS as format}
              <button
                class="format-option"
                class:selected={isFormatSelected(format.value)}
                onclick={() => toggleFormat(format.value)}
              >
                <div class="format-check">
                  {#if isFormatSelected(format.value)}
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M20 6L9 17l-5-5" />
                    </svg>
                  {/if}
                </div>
                <div class="format-info">
                  <span class="format-label">{format.label}</span>
                  <span class="format-description">{format.description}</span>
                </div>
              </button>
            {/each}
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Compression" description="Configure file compression settings">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.output.compression.enabled}
              label="Enable Compression"
              description="Compress output files to reduce disk usage"
            />

            {#if $config.output.compression.enabled}
              <div class="form-grid">
                <FormGroup
                  label="Algorithm"
                  htmlFor="compression-algo"
                  helpText="Compression algorithm to use"
                >
                  {#snippet children()}
                    <select id="compression-algo" bind:value={$config.output.compression.algorithm}>
                      {#each COMPRESSION_ALGORITHMS as algo}
                        <option value={algo.value}>{algo.label} - {algo.description}</option>
                      {/each}
                    </select>
                  {/snippet}
                </FormGroup>

                <FormGroup
                  label="Compression Level"
                  htmlFor="compression-level"
                  helpText="Higher = smaller files, slower (1-9)"
                  error={getError('output.compression.level')}
                >
                  {#snippet children()}
                    <div class="slider-with-value">
                      <input
                        type="range"
                        id="compression-level"
                        bind:value={$config.output.compression.level}
                        min="1"
                        max="9"
                        step="1"
                      />
                      <span class="slider-value">{$config.output.compression.level}</span>
                    </div>
                  {/snippet}
                </FormGroup>
              </div>
            {/if}
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Performance" description="Batch size and write settings">
        {#snippet children()}
          <div class="form-grid">
            <FormGroup
              label="Batch Size"
              htmlFor="batch-size"
              helpText="Records per write batch (affects memory)"
              error={getError('output.batch_size')}
            >
              {#snippet children()}
                <select id="batch-size" bind:value={$config.output.batch_size}>
                  <option value={10000}>10,000</option>
                  <option value={50000}>50,000</option>
                  <option value={100000}>100,000</option>
                  <option value={250000}>250,000</option>
                  <option value={500000}>500,000</option>
                </select>
              {/snippet}
            </FormGroup>
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="SAP Formats" description="Include SAP-specific output formats">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.output.include_acdoca}
              label="Include ACDOCA Format"
              description="Generate SAP HANA Universal Journal format"
            />

            <Toggle
              bind:checked={$config.output.include_bseg}
              label="Include BSEG Format"
              description="Generate SAP BSEG document segment format"
            />
          </div>
        {/snippet}
      </FormSection>

      <FormSection title="Partitioning" description="Split output files by dimensions">
        {#snippet children()}
          <div class="form-stack">
            <Toggle
              bind:checked={$config.output.partition_by_period}
              label="Partition by Fiscal Period"
              description="Create separate files for each month"
            />

            <Toggle
              bind:checked={$config.output.partition_by_company}
              label="Partition by Company Code"
              description="Create separate files for each company"
            />
          </div>
        {/snippet}
      </FormSection>

      <div class="output-preview">
        <h3>Output Preview</h3>
        <div class="preview-path">
          <code>
            {$config.output.output_directory}/
            {#if $config.output.partition_by_company}[company]/company/code/{/if}
            {#if $config.output.partition_by_period}[period]/YYYY-MM/{/if}
            journal_entries.{$config.output.formats[0] ?? 'parquet'}
            {#if $config.output.compression.enabled}.{$config.output.compression.algorithm}{/if}
          </code>
        </div>
        <div class="preview-files">
          <span class="preview-label">Files generated:</span>
          <ul>
            <li>journal_entries.* - Main journal entry data</li>
            {#if $config.output.include_acdoca}
              <li>acdoca.* - SAP HANA Universal Journal</li>
            {/if}
            {#if $config.output.include_bseg}
              <li>bseg.* - SAP Document Segments</li>
            {/if}
            <li>chart_of_accounts.* - CoA master data</li>
            <li>vendors.*, customers.*, materials.* - Entity master data</li>
          </ul>
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

  .mode-selector {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
  }

  .mode-option {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-4);
    background: var(--color-background);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-lg);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: center;
  }

  .mode-option:hover {
    border-color: var(--color-accent);
  }

  .mode-option.selected {
    border-color: var(--color-accent);
    background-color: rgba(99, 102, 241, 0.05);
  }

  .mode-label {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .mode-description {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .format-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: var(--space-3);
  }

  .format-option {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3);
    background: var(--color-background);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
  }

  .format-option:hover {
    border-color: var(--color-accent);
  }

  .format-option.selected {
    border-color: var(--color-accent);
    background-color: rgba(99, 102, 241, 0.05);
  }

  .format-check {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 2px solid var(--color-border);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
    color: var(--color-accent);
  }

  .format-option.selected .format-check {
    border-color: var(--color-accent);
    background-color: var(--color-accent);
    color: white;
  }

  .format-check svg {
    width: 14px;
    height: 14px;
  }

  .format-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .format-label {
    font-weight: 600;
    font-size: 0.875rem;
    color: var(--color-text-primary);
  }

  .format-description {
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
  }

  .slider-with-value input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
    border: 2px solid var(--color-surface);
    box-shadow: var(--shadow-sm);
  }

  .slider-value {
    font-family: var(--font-mono);
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    min-width: 24px;
    text-align: center;
  }

  .output-preview {
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .output-preview h3 {
    font-size: 0.875rem;
    font-weight: 600;
    margin-bottom: var(--space-3);
  }

  .preview-path {
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    margin-bottom: var(--space-4);
  }

  .preview-path code {
    font-size: 0.8125rem;
    color: var(--color-accent);
    word-break: break-all;
  }

  .preview-files {
    font-size: 0.8125rem;
  }

  .preview-label {
    font-weight: 500;
    color: var(--color-text-secondary);
    display: block;
    margin-bottom: var(--space-2);
  }

  .preview-files ul {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    color: var(--color-text-secondary);
  }

  .preview-files li {
    padding-left: var(--space-3);
    position: relative;
  }

  .preview-files li::before {
    content: '•';
    position: absolute;
    left: 0;
    color: var(--color-accent);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-10);
    color: var(--color-text-secondary);
  }

  @media (max-width: 768px) {
    .form-grid {
      grid-template-columns: 1fr;
    }

    .mode-selector {
      grid-template-columns: 1fr;
    }

    .format-grid {
      grid-template-columns: 1fr;
    }

    .page-header {
      flex-direction: column;
      gap: var(--space-4);
    }
  }
</style>
