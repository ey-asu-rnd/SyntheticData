/**
 * Tests for utility functions.
 */
import { describe, it, expect, vi } from 'vitest';
import {
  formatNumber,
  formatRate,
  calculateAnomalyRate,
  formatDuration,
  formatUptime,
  validateEntryCount,
  debounce,
} from './utils';

describe('formatNumber', () => {
  it('should format millions with M suffix', () => {
    expect(formatNumber(1_000_000)).toBe('1.00M');
    expect(formatNumber(1_500_000)).toBe('1.50M');
    expect(formatNumber(10_000_000)).toBe('10.00M');
  });

  it('should format thousands with K suffix', () => {
    expect(formatNumber(1_000)).toBe('1.0K');
    expect(formatNumber(1_500)).toBe('1.5K');
    expect(formatNumber(10_000)).toBe('10.0K');
    expect(formatNumber(999_999)).toBe('1000.0K');
  });

  it('should format small numbers with locale string', () => {
    expect(formatNumber(0)).toBe('0');
    expect(formatNumber(1)).toBe('1');
    expect(formatNumber(999)).toBe('999');
  });

  it('should handle edge cases', () => {
    expect(formatNumber(1_000_000 - 1)).toBe('1000.0K');
    expect(formatNumber(1_000 - 1)).toBe('999');
  });
});

describe('formatRate', () => {
  it('should format rate with one decimal place', () => {
    expect(formatRate(0)).toBe('0.0');
    expect(formatRate(1.23456)).toBe('1.2');
    expect(formatRate(100.99)).toBe('101.0');
  });

  it('should handle whole numbers', () => {
    expect(formatRate(50)).toBe('50.0');
  });
});

describe('calculateAnomalyRate', () => {
  it('should calculate correct percentage', () => {
    expect(calculateAnomalyRate(10, 100)).toBe('10.00');
    expect(calculateAnomalyRate(1, 1000)).toBe('0.10');
    expect(calculateAnomalyRate(50, 200)).toBe('25.00');
  });

  it('should handle zero total', () => {
    expect(calculateAnomalyRate(0, 0)).toBe('0.00');
    expect(calculateAnomalyRate(10, 0)).toBe('0.00');
  });

  it('should handle zero anomalies', () => {
    expect(calculateAnomalyRate(0, 100)).toBe('0.00');
  });
});

describe('formatDuration', () => {
  it('should format milliseconds', () => {
    expect(formatDuration(0)).toBe('0ms');
    expect(formatDuration(500)).toBe('500ms');
    expect(formatDuration(999)).toBe('999ms');
  });

  it('should format seconds', () => {
    expect(formatDuration(1000)).toBe('1.0s');
    expect(formatDuration(1500)).toBe('1.5s');
    expect(formatDuration(30000)).toBe('30.0s');
  });

  it('should format minutes and seconds', () => {
    expect(formatDuration(60000)).toBe('1m 0s');
    expect(formatDuration(90000)).toBe('1m 30s');
    expect(formatDuration(120000)).toBe('2m 0s');
  });
});

describe('formatUptime', () => {
  it('should format seconds', () => {
    expect(formatUptime(0)).toBe('0s');
    expect(formatUptime(30)).toBe('30s');
    expect(formatUptime(59)).toBe('59s');
  });

  it('should format minutes and seconds', () => {
    expect(formatUptime(60)).toBe('1m 0s');
    expect(formatUptime(90)).toBe('1m 30s');
    expect(formatUptime(3599)).toBe('59m 59s');
  });

  it('should format hours and minutes', () => {
    expect(formatUptime(3600)).toBe('1h 0m');
    expect(formatUptime(5400)).toBe('1h 30m');
    expect(formatUptime(86400)).toBe('24h 0m');
  });
});

describe('validateEntryCount', () => {
  it('should accept valid entry counts', () => {
    expect(validateEntryCount(1)).toEqual({ valid: true });
    expect(validateEntryCount(1000)).toEqual({ valid: true });
    expect(validateEntryCount(1_000_000)).toEqual({ valid: true });
  });

  it('should reject entry count less than 1', () => {
    const result = validateEntryCount(0);
    expect(result.valid).toBe(false);
    expect(result.message).toContain('at least 1');
  });

  it('should reject negative entry count', () => {
    const result = validateEntryCount(-1);
    expect(result.valid).toBe(false);
    expect(result.message).toContain('at least 1');
  });

  it('should reject entry count over 1 million', () => {
    const result = validateEntryCount(1_000_001);
    expect(result.valid).toBe(false);
    expect(result.message).toContain('1,000,000');
  });
});

describe('debounce', () => {
  it('should debounce function calls', async () => {
    vi.useFakeTimers();

    const fn = vi.fn();
    const debouncedFn = debounce(fn, 100);

    debouncedFn();
    debouncedFn();
    debouncedFn();

    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });

  it('should pass arguments to the function', async () => {
    vi.useFakeTimers();

    const fn = vi.fn();
    const debouncedFn = debounce(fn, 100);

    debouncedFn('arg1', 'arg2');

    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledWith('arg1', 'arg2');

    vi.useRealTimers();
  });

  it('should reset timer on each call', async () => {
    vi.useFakeTimers();

    const fn = vi.fn();
    const debouncedFn = debounce(fn, 100);

    debouncedFn();
    vi.advanceTimersByTime(50);

    debouncedFn();
    vi.advanceTimersByTime(50);

    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(50);

    expect(fn).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });
});
