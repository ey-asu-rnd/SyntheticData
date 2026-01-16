/**
 * Utility functions for the synth-ui application.
 */

/**
 * Format a large number with K/M suffixes.
 */
export function formatNumber(num: number): string {
  if (num >= 1_000_000) {
    return (num / 1_000_000).toFixed(2) + 'M';
  } else if (num >= 1_000) {
    return (num / 1_000).toFixed(1) + 'K';
  }
  return num.toLocaleString();
}

/**
 * Format a rate with one decimal place.
 */
export function formatRate(rate: number): string {
  return rate.toFixed(1);
}

/**
 * Calculate anomaly rate as a percentage.
 */
export function calculateAnomalyRate(
  anomalies: number,
  total: number
): string {
  if (total === 0) return '0.00';
  return ((anomalies / total) * 100).toFixed(2);
}

/**
 * Format duration in milliseconds to a human-readable string.
 */
export function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  } else if (ms < 60000) {
    return `${(ms / 1000).toFixed(1)}s`;
  } else {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  }
}

/**
 * Format uptime in seconds to a human-readable string.
 */
export function formatUptime(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  } else if (seconds < 3600) {
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${minutes}m ${secs}s`;
  } else {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  }
}

/**
 * Validate entry count is within bounds.
 */
export function validateEntryCount(count: number): { valid: boolean; message?: string } {
  if (count < 1) {
    return { valid: false, message: 'Entry count must be at least 1' };
  }
  if (count > 1_000_000) {
    return { valid: false, message: 'Entry count cannot exceed 1,000,000' };
  }
  return { valid: true };
}

/**
 * Debounce a function call.
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}
