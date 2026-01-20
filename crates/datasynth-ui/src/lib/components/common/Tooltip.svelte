<script lang="ts">
  import type { Snippet } from 'svelte';

  /**
   * Tooltip component that displays help text on hover.
   */
  let {
    text,
    position = 'top',
    children,
  }: {
    text: string;
    position?: 'top' | 'bottom' | 'left' | 'right';
    children: Snippet;
  } = $props();

  let visible = $state(false);
</script>

<span
  class="tooltip-wrapper"
  onmouseenter={() => (visible = true)}
  onmouseleave={() => (visible = false)}
  onfocus={() => (visible = true)}
  onblur={() => (visible = false)}
  role="button"
  tabindex="0"
>
  {@render children()}
  {#if visible}
    <span class="tooltip tooltip-{position}">
      {text}
      <span class="tooltip-arrow"></span>
    </span>
  {/if}
</span>

<style>
  .tooltip-wrapper {
    position: relative;
    display: inline-flex;
    cursor: help;
  }

  .tooltip {
    position: absolute;
    z-index: 1000;
    padding: var(--space-2) var(--space-3);
    font-size: 0.75rem;
    font-weight: 400;
    line-height: 1.4;
    color: white;
    background-color: var(--color-text-primary);
    border-radius: var(--radius-md);
    white-space: nowrap;
    max-width: 250px;
    white-space: normal;
    box-shadow: var(--shadow-lg);
  }

  .tooltip-arrow {
    position: absolute;
    width: 8px;
    height: 8px;
    background-color: var(--color-text-primary);
    transform: rotate(45deg);
  }

  .tooltip-top {
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
  }

  .tooltip-top .tooltip-arrow {
    bottom: -4px;
    left: 50%;
    margin-left: -4px;
  }

  .tooltip-bottom {
    top: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
  }

  .tooltip-bottom .tooltip-arrow {
    top: -4px;
    left: 50%;
    margin-left: -4px;
  }

  .tooltip-left {
    right: calc(100% + 8px);
    top: 50%;
    transform: translateY(-50%);
  }

  .tooltip-left .tooltip-arrow {
    right: -4px;
    top: 50%;
    margin-top: -4px;
  }

  .tooltip-right {
    left: calc(100% + 8px);
    top: 50%;
    transform: translateY(-50%);
  }

  .tooltip-right .tooltip-arrow {
    left: -4px;
    top: 50%;
    margin-top: -4px;
  }
</style>
