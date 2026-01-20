/**
 * Tests for FormGroup component.
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import FormGroupTestWrapper from './FormGroupTestWrapper.svelte';

describe('FormGroup', () => {
	describe('rendering', () => {
		it('should render label text', () => {
			render(FormGroupTestWrapper, { props: { label: 'Username' } });

			expect(screen.getByText('Username')).toBeTruthy();
		});

		it('should render children content', () => {
			render(FormGroupTestWrapper, { props: { label: 'Test' } });

			const childInput = screen.getByTestId('child-input');
			expect(childInput).toBeTruthy();
		});

		it('should associate label with input via htmlFor', () => {
			render(FormGroupTestWrapper, {
				props: { label: 'Username', htmlFor: 'username-input' },
			});

			const label = screen.getByText('Username');
			expect(label.getAttribute('for')).toBe('username-input');
		});
	});

	describe('required indicator', () => {
		it('should not show required indicator by default', () => {
			render(FormGroupTestWrapper, { props: { label: 'Username' } });

			const requiredIndicator = screen.queryByText('*');
			expect(requiredIndicator).toBeNull();
		});

		it('should show required indicator when required=true', () => {
			render(FormGroupTestWrapper, {
				props: { label: 'Username', required: true },
			});

			const requiredIndicator = screen.getByText('*');
			expect(requiredIndicator).toBeTruthy();
			expect(requiredIndicator.classList.contains('required')).toBe(true);
		});
	});

	describe('help text', () => {
		it('should display help text when provided', () => {
			render(FormGroupTestWrapper, {
				props: { label: 'Username', helpText: 'Enter your username here' },
			});

			expect(screen.getByText('Enter your username here')).toBeTruthy();
		});

		it('should not display help text when not provided', () => {
			const { container } = render(FormGroupTestWrapper, {
				props: { label: 'Username' },
			});

			const helpText = container.querySelector('.help-text');
			expect(helpText).toBeNull();
		});

		it('should have help-text class', () => {
			const { container } = render(FormGroupTestWrapper, {
				props: { label: 'Username', helpText: 'Help text here' },
			});

			const helpText = container.querySelector('.help-text');
			expect(helpText).toBeTruthy();
			expect(helpText?.textContent).toBe('Help text here');
		});
	});

	describe('error state', () => {
		it('should display error message when provided', () => {
			render(FormGroupTestWrapper, {
				props: { label: 'Username', error: 'Username is required' },
			});

			expect(screen.getByText('Username is required')).toBeTruthy();
		});

		it('should have error-text class on error message', () => {
			const { container } = render(FormGroupTestWrapper, {
				props: { label: 'Username', error: 'Username is required' },
			});

			const errorText = container.querySelector('.error-text');
			expect(errorText).toBeTruthy();
			expect(errorText?.textContent).toBe('Username is required');
		});

		it('should apply has-error class when error is present', () => {
			const { container } = render(FormGroupTestWrapper, {
				props: { label: 'Username', error: 'Error message' },
			});

			const formGroup = container.querySelector('.form-group');
			expect(formGroup?.classList.contains('has-error')).toBe(true);
		});

		it('should not apply has-error class when no error', () => {
			const { container } = render(FormGroupTestWrapper, {
				props: { label: 'Username' },
			});

			const formGroup = container.querySelector('.form-group');
			expect(formGroup?.classList.contains('has-error')).toBe(false);
		});

		it('should show error message instead of help text when both provided', () => {
			render(FormGroupTestWrapper, {
				props: {
					label: 'Username',
					helpText: 'Enter your username',
					error: 'Username is required',
				},
			});

			// Error should be visible
			expect(screen.getByText('Username is required')).toBeTruthy();

			// Help text should not be visible
			expect(screen.queryByText('Enter your username')).toBeNull();
		});
	});

	describe('structure', () => {
		it('should have form-group wrapper class', () => {
			const { container } = render(FormGroupTestWrapper, {
				props: { label: 'Test' },
			});

			const formGroup = container.querySelector('.form-group');
			expect(formGroup).toBeTruthy();
		});

		it('should have input-wrapper for children', () => {
			const { container } = render(FormGroupTestWrapper, {
				props: { label: 'Test' },
			});

			const inputWrapper = container.querySelector('.input-wrapper');
			expect(inputWrapper).toBeTruthy();
		});

		it('should render label as a label element', () => {
			const { container } = render(FormGroupTestWrapper, {
				props: { label: 'Test Label' },
			});

			const label = container.querySelector('label');
			expect(label).toBeTruthy();
			expect(label?.textContent?.includes('Test Label')).toBe(true);
		});
	});
});
