/**
 * Tests for FormSection component.
 */
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import FormSectionTestWrapper from './FormSectionTestWrapper.svelte';

describe('FormSection', () => {
	describe('rendering', () => {
		it('should render title', () => {
			render(FormSectionTestWrapper, { props: { title: 'Global Settings' } });

			expect(screen.getByText('Global Settings')).toBeTruthy();
		});

		it('should render description when provided', () => {
			render(FormSectionTestWrapper, {
				props: {
					title: 'Settings',
					description: 'Configure your settings here',
				},
			});

			expect(screen.getByText('Configure your settings here')).toBeTruthy();
		});

		it('should not render description element when not provided', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings' },
			});

			const description = container.querySelector('.section-description');
			expect(description).toBeNull();
		});

		it('should render children content', () => {
			render(FormSectionTestWrapper, {
				props: { title: 'Test', childContent: 'Child content here' },
			});

			expect(screen.getByTestId('section-content')).toBeTruthy();
			expect(screen.getByText('Child content here')).toBeTruthy();
		});
	});

	describe('collapsible behavior', () => {
		it('should not show collapse icon when not collapsible', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: false },
			});

			const collapseIcon = container.querySelector('.collapse-icon');
			expect(collapseIcon).toBeNull();
		});

		it('should show collapse icon when collapsible', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true },
			});

			const collapseIcon = container.querySelector('.collapse-icon');
			expect(collapseIcon).toBeTruthy();
		});

		it('should have collapsible class on header when collapsible', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true },
			});

			const header = container.querySelector('.section-header');
			expect(header?.classList.contains('collapsible')).toBe(true);
		});

		it('should not have collapsible class on header when not collapsible', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: false },
			});

			const header = container.querySelector('.section-header');
			expect(header?.classList.contains('collapsible')).toBe(false);
		});

		it('should disable header button when not collapsible', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: false },
			});

			const headerButton = container.querySelector('.section-header') as HTMLButtonElement;
			expect(headerButton.disabled).toBe(true);
		});

		it('should enable header button when collapsible', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true },
			});

			const headerButton = container.querySelector('.section-header') as HTMLButtonElement;
			expect(headerButton.disabled).toBe(false);
		});
	});

	describe('collapse state', () => {
		it('should start expanded by default', () => {
			render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true },
			});

			const content = screen.getByTestId('section-content');
			expect(content).toBeTruthy();
		});

		it('should start collapsed when collapsed=true', () => {
			render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true, collapsed: true },
			});

			const content = screen.queryByTestId('section-content');
			expect(content).toBeNull();
		});

		it('should toggle content on header click when collapsible', async () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true, collapsed: false },
			});

			// Content should be visible initially
			expect(screen.getByTestId('section-content')).toBeTruthy();

			// Click the header
			const headerButton = container.querySelector('.section-header') as HTMLButtonElement;
			await fireEvent.click(headerButton);

			// Content should be hidden
			expect(screen.queryByTestId('section-content')).toBeNull();
		});

		it('should expand collapsed section on header click', async () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true, collapsed: true },
			});

			// Content should be hidden initially
			expect(screen.queryByTestId('section-content')).toBeNull();

			// Click the header
			const headerButton = container.querySelector('.section-header') as HTMLButtonElement;
			await fireEvent.click(headerButton);

			// Content should be visible
			expect(screen.getByTestId('section-content')).toBeTruthy();
		});

		it('should not toggle on header click when not collapsible', async () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: false },
			});

			// Content should be visible
			expect(screen.getByTestId('section-content')).toBeTruthy();

			// Try to click the header
			const headerButton = container.querySelector('.section-header') as HTMLButtonElement;
			await fireEvent.click(headerButton);

			// Content should still be visible
			expect(screen.getByTestId('section-content')).toBeTruthy();
		});

		it('should have collapsed class on icon when collapsed', async () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true, collapsed: true },
			});

			const collapseIcon = container.querySelector('.collapse-icon');
			expect(collapseIcon?.classList.contains('collapsed')).toBe(true);
		});

		it('should not have collapsed class on icon when expanded', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings', collapsible: true, collapsed: false },
			});

			const collapseIcon = container.querySelector('.collapse-icon');
			expect(collapseIcon?.classList.contains('collapsed')).toBe(false);
		});
	});

	describe('structure', () => {
		it('should have form-section wrapper', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings' },
			});

			const section = container.querySelector('.form-section');
			expect(section).toBeTruthy();
		});

		it('should have section element', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings' },
			});

			const section = container.querySelector('section');
			expect(section).toBeTruthy();
		});

		it('should have section-header as a button', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings' },
			});

			const header = container.querySelector('button.section-header');
			expect(header).toBeTruthy();
		});

		it('should have section-content wrapper when expanded', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings' },
			});

			const content = container.querySelector('.section-content');
			expect(content).toBeTruthy();
		});

		it('should have title in h3 element', () => {
			const { container } = render(FormSectionTestWrapper, {
				props: { title: 'Settings' },
			});

			const h3 = container.querySelector('h3.section-title');
			expect(h3).toBeTruthy();
			expect(h3?.textContent).toBe('Settings');
		});
	});
});
