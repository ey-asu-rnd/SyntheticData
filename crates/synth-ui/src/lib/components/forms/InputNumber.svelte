<script lang="ts">
  interface Props {
    id?: string;
    value: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
  }

  let { id, value = $bindable(), min, max, step = 1, disabled = false }: Props = $props();
</script>

<input
  {id}
  type="number"
  bind:value
  {min}
  {max}
  {step}
  {disabled}
  class="input-number"
/>

<style>
  .input-number {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font-size: 0.875rem;
    font-family: var(--font-mono);
    color: var(--color-text-primary);
    background-color: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast);
  }

  .input-number:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .input-number:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Hide spinner buttons */
  .input-number::-webkit-inner-spin-button,
  .input-number::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .input-number {
    -moz-appearance: textfield;
  }
</style>
