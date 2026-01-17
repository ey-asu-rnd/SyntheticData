<script lang="ts">
  import type { Snippet } from 'svelte';

  /**
   * Form group with label, input slot, and optional help text.
   */
  let {
    label,
    htmlFor = '',
    helpText = '',
    error = '',
    required = false,
    children,
  }: {
    label: string;
    htmlFor?: string;
    helpText?: string;
    error?: string;
    required?: boolean;
    children: Snippet;
  } = $props();
</script>

<div class="form-group" class:has-error={!!error}>
  <label for={htmlFor}>
    {label}
    {#if required}
      <span class="required">*</span>
    {/if}
  </label>
  <div class="input-wrapper">
    {@render children()}
  </div>
  {#if error}
    <p class="error-text">{error}</p>
  {:else if helpText}
    <p class="help-text">{helpText}</p>
  {/if}
</div>

<style>
  .form-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  label {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .required {
    color: var(--color-danger);
  }

  .input-wrapper {
    display: flex;
    flex-direction: column;
  }

  .input-wrapper :global(input),
  .input-wrapper :global(select),
  .input-wrapper :global(textarea) {
    width: 100%;
  }

  .has-error .input-wrapper :global(input),
  .has-error .input-wrapper :global(select),
  .has-error .input-wrapper :global(textarea) {
    border-color: var(--color-danger);
  }

  .help-text {
    font-size: 0.75rem;
    color: var(--color-text-muted);
    margin: 0;
  }

  .error-text {
    font-size: 0.75rem;
    color: var(--color-danger);
    margin: 0;
  }
</style>
