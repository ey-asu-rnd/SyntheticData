/**
 * Tests for config store.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

// Mock Tauri invoke before importing the store
// Note: vi.mock is hoisted, so we use vi.hoisted to get the mock function
const { mockInvoke } = vi.hoisted(() => ({
	mockInvoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: mockInvoke,
}));

// Import after mocking
import {
	configStore,
	createDefaultConfig,
	type GeneratorConfig,
	type ValidationError,
} from './config';

describe('config store', () => {
	beforeEach(() => {
		mockInvoke.mockReset();
		// Reset store state by loading a fresh config
		configStore.set(null);
	});

	afterEach(() => {
		vi.clearAllMocks();
	});

	describe('createDefaultConfig', () => {
		it('should return a valid default configuration', () => {
			const config = createDefaultConfig();

			expect(config).toBeTruthy();
			expect(config.global).toBeTruthy();
			expect(config.companies).toBeTruthy();
			expect(config.companies.length).toBeGreaterThan(0);
		});

		it('should have default industry set to manufacturing', () => {
			const config = createDefaultConfig();

			expect(config.global.industry).toBe('manufacturing');
		});

		it('should have default period_months of 12', () => {
			const config = createDefaultConfig();

			expect(config.global.period_months).toBe(12);
		});

		it('should have at least one company', () => {
			const config = createDefaultConfig();

			expect(config.companies.length).toBe(1);
			expect(config.companies[0].code).toBe('1000');
			expect(config.companies[0].name).toBe('US Manufacturing');
		});

		it('should have fraud disabled by default', () => {
			const config = createDefaultConfig();

			expect(config.fraud.enabled).toBe(false);
		});

		it('should have internal controls disabled by default', () => {
			const config = createDefaultConfig();

			expect(config.internal_controls.enabled).toBe(false);
		});
	});

	describe('load()', () => {
		it('should set loading state while fetching', async () => {
			mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves

			const loadPromise = configStore.load();

			expect(get(configStore.loading)).toBe(true);

			// Clean up by not awaiting the never-resolving promise
		});

		it('should populate config on successful load', async () => {
			const mockConfig = createDefaultConfig();
			mockConfig.global.industry = 'retail';

			mockInvoke.mockResolvedValue({
				success: true,
				config: mockConfig,
				message: '',
			});

			await configStore.load();

			const config = get(configStore.config);
			expect(config?.global.industry).toBe('retail');
		});

		it('should set loading to false after load', async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});

			await configStore.load();

			expect(get(configStore.loading)).toBe(false);
		});

		it('should call Tauri invoke with get_config', async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});

			await configStore.load();

			expect(mockInvoke).toHaveBeenCalledWith('get_config');
		});

		it('should use default config when backend returns no config', async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: null,
				message: '',
			});

			await configStore.load();

			const config = get(configStore.config);
			expect(config).toBeTruthy();
			expect(config?.global.industry).toBe('manufacturing'); // default
		});

		it('should use default config on error', async () => {
			mockInvoke.mockRejectedValue(new Error('Network error'));

			await configStore.load();

			const config = get(configStore.config);
			expect(config).toBeTruthy();
			expect(get(configStore.error)).toBe('Error: Network error');
		});
	});

	describe('save()', () => {
		beforeEach(async () => {
			// Initialize store with a config
			mockInvoke.mockResolvedValueOnce({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});
			await configStore.load();
		});

		it('should set saving state while saving', async () => {
			mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves

			const savePromise = configStore.save();

			expect(get(configStore.saving)).toBe(true);
		});

		it('should call Tauri invoke with set_config', async () => {
			mockInvoke.mockResolvedValue({ success: true, message: '' });

			await configStore.save();

			expect(mockInvoke).toHaveBeenCalledWith('set_config', {
				config: expect.any(Object),
			});
		});

		it('should return true on successful save', async () => {
			mockInvoke.mockResolvedValue({ success: true, message: '' });

			const result = await configStore.save();

			expect(result).toBe(true);
		});

		it('should return false on failed save', async () => {
			mockInvoke.mockResolvedValue({ success: false, message: 'Save failed' });

			const result = await configStore.save();

			expect(result).toBe(false);
		});

		it('should set error on failed save', async () => {
			mockInvoke.mockResolvedValue({ success: false, message: 'Save failed' });

			await configStore.save();

			expect(get(configStore.error)).toBe('Save failed');
		});

		it('should set error on exception', async () => {
			mockInvoke.mockRejectedValue(new Error('Network error'));

			await configStore.save();

			expect(get(configStore.error)).toBe('Error: Network error');
		});

		it('should not save if config has validation errors', async () => {
			// Create an invalid config
			const invalidConfig = createDefaultConfig();
			invalidConfig.global.period_months = 200; // Invalid: > 120
			configStore.set(invalidConfig);

			const result = await configStore.save();

			expect(result).toBe(false);
			// invoke should not be called for invalid config (beyond the initial load)
		});
	});

	describe('isDirty', () => {
		beforeEach(async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});
			await configStore.load();
		});

		it('should be false after initial load', () => {
			expect(get(configStore.isDirty)).toBe(false);
		});

		it('should become true after updateField', () => {
			const newGlobal = { ...createDefaultConfig().global, industry: 'retail' };
			configStore.updateField('global', newGlobal);

			expect(get(configStore.isDirty)).toBe(true);
		});

		it('should become false after reset', () => {
			const newGlobal = { ...createDefaultConfig().global, industry: 'retail' };
			configStore.updateField('global', newGlobal);

			expect(get(configStore.isDirty)).toBe(true);

			configStore.reset();

			expect(get(configStore.isDirty)).toBe(false);
		});

		it('should detect nested changes', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					global: { ...cfg.global, period_months: 24 },
				};
			});

			expect(get(configStore.isDirty)).toBe(true);
		});
	});

	describe('reset()', () => {
		beforeEach(async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});
			await configStore.load();
		});

		it('should revert config to original', () => {
			const originalIndustry = get(configStore.config)?.global.industry;

			// Make changes
			const newGlobal = { ...createDefaultConfig().global, industry: 'healthcare' };
			configStore.updateField('global', newGlobal);

			expect(get(configStore.config)?.global.industry).toBe('healthcare');

			// Reset
			configStore.reset();

			expect(get(configStore.config)?.global.industry).toBe(originalIndustry);
		});

		it('should clear error state', async () => {
			// Trigger an error
			mockInvoke.mockRejectedValueOnce(new Error('Some error'));
			await configStore.save();

			expect(get(configStore.error)).toBeTruthy();

			configStore.reset();

			expect(get(configStore.error)).toBeNull();
		});
	});

	describe('updateField()', () => {
		beforeEach(async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});
			await configStore.load();
		});

		it('should update top-level field', () => {
			const newGlobal = {
				...createDefaultConfig().global,
				industry: 'technology',
			};
			configStore.updateField('global', newGlobal);

			expect(get(configStore.config)?.global.industry).toBe('technology');
		});

		it('should update companies array', () => {
			const newCompanies = [
				{
					code: '2000',
					name: 'New Company',
					currency: 'EUR',
					country: 'DE',
					fiscal_year_variant: 'K4',
					annual_transaction_volume: 'hundred_k',
					volume_weight: 1.0,
				},
			];
			configStore.updateField('companies', newCompanies);

			expect(get(configStore.config)?.companies[0].code).toBe('2000');
		});

		it('should update fraud settings', () => {
			const newFraud = {
				...createDefaultConfig().fraud,
				enabled: true,
				fraud_rate: 0.01,
			};
			configStore.updateField('fraud', newFraud);

			expect(get(configStore.config)?.fraud.enabled).toBe(true);
			expect(get(configStore.config)?.fraud.fraud_rate).toBe(0.01);
		});
	});

	describe('applyPreset()', () => {
		beforeEach(async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});
			await configStore.load();
		});

		it('should apply preset configuration', () => {
			const presetConfig = createDefaultConfig();
			presetConfig.global.industry = 'financial_services';
			presetConfig.global.period_months = 36;

			configStore.applyPreset(presetConfig);

			const config = get(configStore.config);
			expect(config?.global.industry).toBe('financial_services');
			expect(config?.global.period_months).toBe(36);
		});

		it('should make config dirty after applying preset', () => {
			const presetConfig = createDefaultConfig();
			presetConfig.global.industry = 'retail';

			configStore.applyPreset(presetConfig);

			expect(get(configStore.isDirty)).toBe(true);
		});
	});

	describe('validationErrors', () => {
		beforeEach(async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});
			await configStore.load();
		});

		it('should be empty for valid config', () => {
			const errors = get(configStore.validationErrors);
			expect(errors.length).toBe(0);
		});

		it('should detect invalid period_months (>120)', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					global: { ...cfg.global, period_months: 150 },
				};
			});

			const errors = get(configStore.validationErrors);
			expect(errors.some((e) => e.field === 'global.period_months')).toBe(true);
		});

		it('should detect invalid period_months (<1)', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					global: { ...cfg.global, period_months: 0 },
				};
			});

			const errors = get(configStore.validationErrors);
			expect(errors.some((e) => e.field === 'global.period_months')).toBe(true);
		});

		it('should detect invalid start_date format', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					global: { ...cfg.global, start_date: '01-01-2024' }, // Wrong format
				};
			});

			const errors = get(configStore.validationErrors);
			expect(errors.some((e) => e.field === 'global.start_date')).toBe(true);
		});

		it('should detect negative memory limit', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					global: { ...cfg.global, memory_limit_mb: -100 },
				};
			});

			const errors = get(configStore.validationErrors);
			expect(errors.some((e) => e.field === 'global.memory_limit_mb')).toBe(true);
		});

		it('should detect empty companies array', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return { ...cfg, companies: [] };
			});

			const errors = get(configStore.validationErrors);
			expect(errors.some((e) => e.field === 'companies')).toBe(true);
		});

		it('should detect missing company code', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					companies: [{ ...cfg.companies[0], code: '' }],
				};
			});

			const errors = get(configStore.validationErrors);
			expect(errors.some((e) => e.field.includes('companies') && e.field.includes('code'))).toBe(
				true
			);
		});

		it('should detect invalid fraud rate when fraud enabled', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					fraud: { ...cfg.fraud, enabled: true, fraud_rate: 0.5 }, // 50% is too high
				};
			});

			const errors = get(configStore.validationErrors);
			expect(errors.some((e) => e.field === 'fraud.fraud_rate')).toBe(true);
		});

		it('should not validate fraud rate when fraud disabled', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					fraud: { ...cfg.fraud, enabled: false, fraud_rate: 0.5 },
				};
			});

			const errors = get(configStore.validationErrors);
			expect(errors.some((e) => e.field === 'fraud.fraud_rate')).toBe(false);
		});
	});

	describe('isValid', () => {
		beforeEach(async () => {
			mockInvoke.mockResolvedValue({
				success: true,
				config: createDefaultConfig(),
				message: '',
			});
			await configStore.load();
		});

		it('should be true for valid default config', () => {
			expect(get(configStore.isValid)).toBe(true);
		});

		it('should be false when config has errors', () => {
			configStore.update((cfg) => {
				if (!cfg) return cfg;
				return {
					...cfg,
					global: { ...cfg.global, period_months: 200 },
				};
			});

			expect(get(configStore.isValid)).toBe(false);
		});
	});
});
