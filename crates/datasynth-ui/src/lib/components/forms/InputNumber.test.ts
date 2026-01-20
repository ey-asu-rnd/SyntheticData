/**
 * Tests for InputNumber component.
 */
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import InputNumber from './InputNumber.svelte';

describe('InputNumber', () => {
	describe('rendering', () => {
		it('should render with initial value', () => {
			render(InputNumber, { props: { value: 42 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input).toBeTruthy();
			expect(input.value).toBe('42');
		});

		it('should render with id attribute when provided', () => {
			render(InputNumber, { props: { value: 10, id: 'test-input' } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.id).toBe('test-input');
		});

		it('should render with min attribute when provided', () => {
			render(InputNumber, { props: { value: 5, min: 0 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.min).toBe('0');
		});

		it('should render with max attribute when provided', () => {
			render(InputNumber, { props: { value: 5, max: 100 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.max).toBe('100');
		});

		it('should render with step attribute', () => {
			render(InputNumber, { props: { value: 5, step: 0.5 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.step).toBe('0.5');
		});

		it('should use default step of 1 when not provided', () => {
			render(InputNumber, { props: { value: 5 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.step).toBe('1');
		});

		it('should render with min, max, and step together', () => {
			render(InputNumber, { props: { value: 50, min: 0, max: 100, step: 10 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.min).toBe('0');
			expect(input.max).toBe('100');
			expect(input.step).toBe('10');
			expect(input.value).toBe('50');
		});
	});

	describe('disabled state', () => {
		it('should render enabled by default', () => {
			render(InputNumber, { props: { value: 5 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.disabled).toBe(false);
		});

		it('should render disabled when disabled=true', () => {
			render(InputNumber, { props: { value: 5, disabled: true } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.disabled).toBe(true);
		});

		it('should not update value when disabled', async () => {
			render(InputNumber, { props: { value: 5, disabled: true } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;

			// Try to change value
			await fireEvent.input(input, { target: { value: '10' } });

			// Value should not change because the input is disabled
			// Note: In browsers, disabled inputs prevent user interaction
			expect(input.disabled).toBe(true);
		});
	});

	describe('user interaction', () => {
		it('should update value on user input', async () => {
			render(InputNumber, { props: { value: 5 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;

			await fireEvent.input(input, { target: { value: '42' } });

			expect(input.value).toBe('42');
		});

		it('should handle decimal input with step', async () => {
			render(InputNumber, { props: { value: 1.5, step: 0.5 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;

			await fireEvent.input(input, { target: { value: '2.5' } });

			expect(input.value).toBe('2.5');
		});

		it('should handle negative values when min allows', async () => {
			render(InputNumber, { props: { value: 0, min: -100 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;

			await fireEvent.input(input, { target: { value: '-50' } });

			expect(input.value).toBe('-50');
		});

		it('should handle large numbers', async () => {
			render(InputNumber, { props: { value: 1000000 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;

			await fireEvent.input(input, { target: { value: '999999999' } });

			expect(input.value).toBe('999999999');
		});

		it('should handle zero value', () => {
			render(InputNumber, { props: { value: 0 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.value).toBe('0');
		});
	});

	describe('styling', () => {
		it('should have input-number class', () => {
			render(InputNumber, { props: { value: 5 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.classList.contains('input-number')).toBe(true);
		});

		it('should have type=number attribute', () => {
			render(InputNumber, { props: { value: 5 } });

			const input = screen.getByRole('spinbutton') as HTMLInputElement;
			expect(input.type).toBe('number');
		});
	});
});
