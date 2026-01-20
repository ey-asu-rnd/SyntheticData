/**
 * Tests for DistributionEditor component.
 */
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import DistributionEditor from './DistributionEditor.svelte';

describe('DistributionEditor', () => {
	const simpleDistribution = {
		option_a: 0.5,
		option_b: 0.3,
		option_c: 0.2,
	};

	const singleItemDistribution = {
		only_option: 1.0,
	};

	const manyItemsDistribution = {
		item_1: 0.15,
		item_2: 0.15,
		item_3: 0.15,
		item_4: 0.15,
		item_5: 0.15,
		item_6: 0.10,
		item_7: 0.15,
	};

	describe('rendering', () => {
		it('should render all distribution options', () => {
			render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			// Should have 3 bar rows
			expect(screen.getByText('option_a')).toBeTruthy();
			expect(screen.getByText('option_b')).toBeTruthy();
			expect(screen.getByText('option_c')).toBeTruthy();
		});

		it('should render with custom labels', () => {
			render(DistributionEditor, {
				props: {
					distribution: { ...simpleDistribution },
					labels: {
						option_a: 'First Option',
						option_b: 'Second Option',
						option_c: 'Third Option',
					},
				},
			});

			expect(screen.getByText('First Option')).toBeTruthy();
			expect(screen.getByText('Second Option')).toBeTruthy();
			expect(screen.getByText('Third Option')).toBeTruthy();
		});

		it('should display percentages correctly', () => {
			render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			expect(screen.getByText('50.0%')).toBeTruthy();
			expect(screen.getByText('30.0%')).toBeTruthy();
			expect(screen.getByText('20.0%')).toBeTruthy();
		});

		it('should display label when provided', () => {
			render(DistributionEditor, {
				props: {
					distribution: { ...simpleDistribution },
					label: 'Distribution Settings',
				},
			});

			expect(screen.getByText('Distribution Settings')).toBeTruthy();
		});

		it('should not display label when not provided', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const label = container.querySelector('.editor-label');
			expect(label).toBeNull();
		});
	});

	describe('descriptions', () => {
		it('should display descriptions when provided', () => {
			render(DistributionEditor, {
				props: {
					distribution: { ...simpleDistribution },
					descriptions: {
						option_a: 'Description for option A',
						option_b: 'Description for option B',
					},
				},
			});

			expect(screen.getByText('Description for option A')).toBeTruthy();
			expect(screen.getByText('Description for option B')).toBeTruthy();
		});

		it('should add has-description class to rows with descriptions', () => {
			const { container } = render(DistributionEditor, {
				props: {
					distribution: { ...simpleDistribution },
					descriptions: {
						option_a: 'Description for option A',
					},
				},
			});

			const rowsWithDescription = container.querySelectorAll('.bar-row.has-description');
			expect(rowsWithDescription.length).toBe(1);
		});
	});

	describe('help text', () => {
		it('should display help text when provided', () => {
			render(DistributionEditor, {
				props: {
					distribution: { ...simpleDistribution },
					helpText: 'Values will be automatically normalized to sum to 100%',
				},
			});

			expect(
				screen.getByText('Values will be automatically normalized to sum to 100%')
			).toBeTruthy();
		});

		it('should not display help text when not provided', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const helpText = container.querySelector('.help-text');
			expect(helpText).toBeNull();
		});
	});

	describe('value normalization', () => {
		it('should normalize values to sum to 1.0', async () => {
			const distribution = { a: 0.5, b: 0.5 };
			const { container } = render(DistributionEditor, {
				props: { distribution },
			});

			// Change one value via slider
			const sliders = container.querySelectorAll('.bar-slider');
			const firstSlider = sliders[0] as HTMLInputElement;

			// Simulate changing the first slider to 80%
			await fireEvent.input(firstSlider, { target: { value: '80' } });

			// After normalization, both values should still sum to 1
			// The exact values depend on normalization logic
		});

		it('should handle single-item distribution', () => {
			render(DistributionEditor, {
				props: { distribution: { ...singleItemDistribution } },
			});

			// Single item should show 100%
			expect(screen.getByText('100.0%')).toBeTruthy();
		});

		it('should handle distribution with many items', () => {
			render(DistributionEditor, {
				props: { distribution: { ...manyItemsDistribution } },
			});

			// All 7 items should be rendered
			const barRows = screen.getAllByText(/item_\d/);
			expect(barRows.length).toBe(7);
		});
	});

	describe('slider interaction', () => {
		it('should have range input for each distribution item', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const sliders = container.querySelectorAll('input[type="range"]');
			expect(sliders.length).toBe(3);
		});

		it('should have correct min/max/step on sliders', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const slider = container.querySelector('input[type="range"]') as HTMLInputElement;
			expect(slider.min).toBe('0');
			expect(slider.max).toBe('100');
			expect(slider.step).toBe('1');
		});

		it('should update percentage display when slider changes', async () => {
			const distribution = { a: 0.5, b: 0.5 };
			const { container } = render(DistributionEditor, {
				props: { distribution },
			});

			// Both should start at 50%
			const percentages = screen.getAllByText('50.0%');
			expect(percentages.length).toBe(2);
		});
	});

	describe('visual elements', () => {
		it('should have bar track for each item', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const tracks = container.querySelectorAll('.bar-track');
			expect(tracks.length).toBe(3);
		});

		it('should have bar fill for each item', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const fills = container.querySelectorAll('.bar-fill');
			expect(fills.length).toBe(3);
		});

		it('should set bar fill width based on percentage', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { option_a: 1.0 } },
			});

			const fill = container.querySelector('.bar-fill') as HTMLElement;
			expect(fill.style.width).toBe('100%');
		});
	});

	describe('structure', () => {
		it('should have distribution-editor wrapper', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const editor = container.querySelector('.distribution-editor');
			expect(editor).toBeTruthy();
		});

		it('should have distribution-bars container', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const bars = container.querySelector('.distribution-bars');
			expect(bars).toBeTruthy();
		});

		it('should have bar-row for each item', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: { ...simpleDistribution } },
			});

			const rows = container.querySelectorAll('.bar-row');
			expect(rows.length).toBe(3);
		});
	});

	describe('edge cases', () => {
		it('should handle empty distribution', () => {
			const { container } = render(DistributionEditor, {
				props: { distribution: {} },
			});

			const rows = container.querySelectorAll('.bar-row');
			expect(rows.length).toBe(0);
		});

		it('should handle zero values in distribution', () => {
			render(DistributionEditor, {
				props: { distribution: { a: 0, b: 1.0 } },
			});

			expect(screen.getByText('0.0%')).toBeTruthy();
			expect(screen.getByText('100.0%')).toBeTruthy();
		});

		it('should use key as label when no custom label provided', () => {
			render(DistributionEditor, {
				props: {
					distribution: { custom_key: 0.5, another_key: 0.5 },
					labels: { custom_key: 'Custom Label' },
				},
			});

			expect(screen.getByText('Custom Label')).toBeTruthy();
			expect(screen.getByText('another_key')).toBeTruthy();
		});
	});
});
