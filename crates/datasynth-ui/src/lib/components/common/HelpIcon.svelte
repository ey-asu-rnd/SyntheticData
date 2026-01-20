<script lang="ts">
  import Tooltip from './Tooltip.svelte';

  /**
   * Help icon with tooltip - use for inline help in forms.
   */
  let {
    text,
    position = 'top',
  }: {
    text: string;
    position?: 'top' | 'bottom' | 'left' | 'right';
  } = $props();
</script>

<Tooltip {text} {position}>
  <span class="help-icon">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="10" />
      <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
      <line x1="12" y1="17" x2="12.01" y2="17" />
    </svg>
  </span>
</Tooltip>

<style>
  .help-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    color: var(--color-text-muted);
    transition: color var(--transition-fast);
  }

  .help-icon:hover {
    color: var(--color-accent);
  }

  .help-icon svg {
    width: 100%;
    height: 100%;
  }
</style>
