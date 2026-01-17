/**
 * Configuration store with dirty tracking and validation.
 *
 * Manages the application-wide generator configuration state.
 */
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types matching the backend schema
export interface CompanyConfig {
  code: string;
  name: string;
  currency: string;
  country: string;
  fiscal_year_variant: string;
  annual_transaction_volume: string;
  volume_weight: number;
}

export interface GlobalConfig {
  seed: number | null;
  industry: string;
  start_date: string;
  period_months: number;
  group_currency: string;
  parallel: boolean;
  worker_threads: number;
  memory_limit_mb: number;
}

export interface ChartOfAccountsConfig {
  complexity: string;
  industry_specific: boolean;
  min_hierarchy_depth: number;
  max_hierarchy_depth: number;
}

export interface TransactionConfig {
  line_item_distribution: Record<string, number>;
  amount_distribution: AmountDistribution;
  source_distribution: Record<string, number>;
  seasonality: SeasonalityConfig;
}

export interface AmountDistribution {
  min_amount: number;
  max_amount: number;
  lognormal_mu: number;
  lognormal_sigma: number;
  round_number_probability: number;
  nice_number_probability: number;
  benford_compliance: boolean;
}

export interface SeasonalityConfig {
  month_end_spike: boolean;
  month_end_multiplier: number;
  quarter_end_spike: boolean;
  quarter_end_multiplier: number;
  year_end_spike: boolean;
  year_end_multiplier: number;
  day_of_week_patterns: boolean;
}

export interface FraudTypeDistribution {
  suspense_account_abuse: number;
  fictitious_transaction: number;
  revenue_manipulation: number;
  expense_capitalization: number;
  split_transaction: number;
  timing_anomaly: number;
  unauthorized_access: number;
  duplicate_payment: number;
}

export interface FraudConfig {
  enabled: boolean;
  fraud_rate: number;
  fraud_type_distribution: FraudTypeDistribution;
  clustering_enabled: boolean;
  clustering_factor: number;
  approval_thresholds: number[];
}

export interface InternalControlsConfig {
  enabled: boolean;
  exception_rate: number;
  sod_violation_rate: number;
  export_control_master_data: boolean;
  sox_materiality_threshold: number;
}

export interface CompressionConfig {
  enabled: boolean;
  algorithm: string;
  level: number;
}

export interface OutputConfig {
  mode: string;
  output_directory: string;
  formats: string[];
  compression: CompressionConfig;
  batch_size: number;
  include_acdoca: boolean;
  include_bseg: boolean;
  partition_by_period: boolean;
  partition_by_company: boolean;
}

export interface MasterDataConfig {
  vendors: EntityDistribution;
  customers: EntityDistribution;
  materials: EntityDistribution;
  assets: EntityDistribution;
  employees: EntityDistribution;
}

export interface EntityDistribution {
  count: number;
  distribution: Record<string, number>;
}

export interface DocumentLineCountDistribution {
  min_lines: number;
  max_lines: number;
  mode_lines: number;
}

export interface P2PFlowConfig {
  enabled: boolean;
  three_way_match_rate: number;
  partial_delivery_rate: number;
  price_variance_rate: number;
  max_price_variance_percent: number;
  quantity_variance_rate: number;
  average_po_to_gr_days: number;
  average_gr_to_invoice_days: number;
  average_invoice_to_payment_days: number;
  line_count_distribution: DocumentLineCountDistribution;
}

export interface CashDiscountConfig {
  eligible_rate: number;
  taken_rate: number;
  discount_percent: number;
  discount_days: number;
}

export interface O2CFlowConfig {
  enabled: boolean;
  credit_check_failure_rate: number;
  partial_shipment_rate: number;
  return_rate: number;
  bad_debt_rate: number;
  average_so_to_delivery_days: number;
  average_delivery_to_invoice_days: number;
  average_invoice_to_receipt_days: number;
  line_count_distribution: DocumentLineCountDistribution;
  cash_discount: CashDiscountConfig;
}

export interface DocumentFlowConfig {
  p2p: P2PFlowConfig;
  o2c: O2CFlowConfig;
  generate_document_references: boolean;
  export_flow_graph: boolean;
}

export interface BalanceConfig {
  generate_opening_balance: boolean;
  coherence_check: boolean;
  opening_balance_date: string;
}

// Full generator config
export interface GeneratorConfig {
  global: GlobalConfig;
  companies: CompanyConfig[];
  chart_of_accounts: ChartOfAccountsConfig;
  transactions: TransactionConfig;
  output: OutputConfig;
  fraud: FraudConfig;
  internal_controls: InternalControlsConfig;
  master_data: MasterDataConfig;
  document_flows: DocumentFlowConfig;
  balance: BalanceConfig;
}

// Default configuration
export function createDefaultConfig(): GeneratorConfig {
  return {
    global: {
      seed: null,
      industry: 'manufacturing',
      start_date: '2024-01-01',
      period_months: 12,
      group_currency: 'USD',
      parallel: true,
      worker_threads: 0,
      memory_limit_mb: 0,
    },
    companies: [{
      code: '1000',
      name: 'US Manufacturing',
      currency: 'USD',
      country: 'US',
      fiscal_year_variant: 'K4',
      annual_transaction_volume: 'hundred_k',
      volume_weight: 1.0,
    }],
    chart_of_accounts: {
      complexity: 'medium',
      industry_specific: true,
      min_hierarchy_depth: 2,
      max_hierarchy_depth: 5,
    },
    transactions: {
      line_item_distribution: {
        '2': 0.61,
        '3': 0.06,
        '4': 0.17,
        '5': 0.03,
        '6': 0.03,
        '7-9': 0.04,
        '10-99': 0.06,
      },
      amount_distribution: {
        min_amount: 0.01,
        max_amount: 100000000,
        lognormal_mu: 7.0,
        lognormal_sigma: 2.5,
        round_number_probability: 0.25,
        nice_number_probability: 0.15,
        benford_compliance: true,
      },
      source_distribution: {
        manual: 0.1,
        interface: 0.3,
        batch: 0.4,
        recurring: 0.2,
      },
      seasonality: {
        month_end_spike: true,
        month_end_multiplier: 2.5,
        quarter_end_spike: true,
        quarter_end_multiplier: 4.0,
        year_end_spike: true,
        year_end_multiplier: 6.0,
        day_of_week_patterns: true,
      },
    },
    output: {
      mode: 'flat_file',
      output_directory: './output',
      formats: ['parquet'],
      compression: {
        enabled: true,
        algorithm: 'zstd',
        level: 3,
      },
      batch_size: 100000,
      include_acdoca: true,
      include_bseg: false,
      partition_by_period: true,
      partition_by_company: false,
    },
    fraud: {
      enabled: false,
      fraud_rate: 0.005,
      fraud_type_distribution: {
        suspense_account_abuse: 0.25,
        fictitious_transaction: 0.15,
        revenue_manipulation: 0.10,
        expense_capitalization: 0.10,
        split_transaction: 0.15,
        timing_anomaly: 0.10,
        unauthorized_access: 0.10,
        duplicate_payment: 0.05,
      },
      clustering_enabled: false,
      clustering_factor: 3.0,
      approval_thresholds: [1000, 5000, 10000, 25000, 50000, 100000],
    },
    internal_controls: {
      enabled: false,
      exception_rate: 0.02,
      sod_violation_rate: 0.01,
      export_control_master_data: true,
      sox_materiality_threshold: 10000,
    },
    master_data: {
      vendors: { count: 100, distribution: {} },
      customers: { count: 100, distribution: {} },
      materials: { count: 200, distribution: {} },
      assets: { count: 50, distribution: {} },
      employees: { count: 20, distribution: {} },
    },
    document_flows: {
      p2p: {
        enabled: true,
        three_way_match_rate: 0.95,
        partial_delivery_rate: 0.15,
        price_variance_rate: 0.08,
        max_price_variance_percent: 0.05,
        quantity_variance_rate: 0.05,
        average_po_to_gr_days: 14,
        average_gr_to_invoice_days: 5,
        average_invoice_to_payment_days: 30,
        line_count_distribution: {
          min_lines: 1,
          max_lines: 20,
          mode_lines: 3,
        },
      },
      o2c: {
        enabled: true,
        credit_check_failure_rate: 0.02,
        partial_shipment_rate: 0.10,
        return_rate: 0.03,
        bad_debt_rate: 0.01,
        average_so_to_delivery_days: 7,
        average_delivery_to_invoice_days: 1,
        average_invoice_to_receipt_days: 45,
        line_count_distribution: {
          min_lines: 1,
          max_lines: 20,
          mode_lines: 3,
        },
        cash_discount: {
          eligible_rate: 0.30,
          taken_rate: 0.60,
          discount_percent: 0.02,
          discount_days: 10,
        },
      },
      generate_document_references: true,
      export_flow_graph: false,
    },
    balance: {
      generate_opening_balance: true,
      coherence_check: true,
      opening_balance_date: '2024-01-01',
    },
  };
}

// Validation errors
export interface ValidationError {
  field: string;
  message: string;
}

// Store state
function createConfigStore() {
  // The current configuration being edited
  const config = writable<GeneratorConfig | null>(null);

  // The original (saved) configuration for dirty tracking
  const originalConfig = writable<GeneratorConfig | null>(null);

  // Loading and saving states
  const loading = writable(false);
  const saving = writable(false);
  const error = writable<string | null>(null);

  // Derived: is the config dirty (has unsaved changes)?
  const isDirty = derived(
    [config, originalConfig],
    ([$config, $originalConfig]) => {
      if (!$config || !$originalConfig) return false;
      return JSON.stringify($config) !== JSON.stringify($originalConfig);
    }
  );

  // Derived: validation errors
  const validationErrors = derived(config, ($config) => {
    if (!$config) return [];
    return validateConfig($config);
  });

  // Derived: is the config valid?
  const isValid = derived(validationErrors, ($errors) => $errors.length === 0);

  // Load configuration from backend
  async function load(): Promise<void> {
    loading.set(true);
    error.set(null);

    try {
      const response = await invoke<{ success: boolean; config: GeneratorConfig | null; message: string }>('get_config');
      if (response.success && response.config) {
        config.set(response.config);
        originalConfig.set(JSON.parse(JSON.stringify(response.config)));
      } else {
        // Use default config if backend doesn't have one
        const defaultCfg = createDefaultConfig();
        config.set(defaultCfg);
        originalConfig.set(JSON.parse(JSON.stringify(defaultCfg)));
      }
    } catch (e) {
      error.set(String(e));
      // Still provide default config on error
      const defaultCfg = createDefaultConfig();
      config.set(defaultCfg);
      originalConfig.set(JSON.parse(JSON.stringify(defaultCfg)));
    } finally {
      loading.set(false);
    }
  }

  // Save configuration to backend
  async function save(): Promise<boolean> {
    const currentConfig = get(config);
    if (!currentConfig) return false;

    // Validate first
    const errors = validateConfig(currentConfig);
    if (errors.length > 0) {
      error.set(errors.map(e => e.message).join('; '));
      return false;
    }

    saving.set(true);
    error.set(null);

    try {
      const response = await invoke<{ success: boolean; message: string }>('set_config', { config: currentConfig });
      if (response.success) {
        originalConfig.set(JSON.parse(JSON.stringify(currentConfig)));
        return true;
      } else {
        error.set(response.message);
        return false;
      }
    } catch (e) {
      error.set(String(e));
      return false;
    } finally {
      saving.set(false);
    }
  }

  // Reset to original (discard changes)
  function reset(): void {
    const original = get(originalConfig);
    if (original) {
      config.set(JSON.parse(JSON.stringify(original)));
    }
    error.set(null);
  }

  // Apply a preset configuration
  function applyPreset(preset: GeneratorConfig): void {
    config.set(JSON.parse(JSON.stringify(preset)));
  }

  // Update a specific field
  function updateField<K extends keyof GeneratorConfig>(section: K, value: GeneratorConfig[K]): void {
    config.update(cfg => {
      if (!cfg) return cfg;
      return { ...cfg, [section]: value };
    });
  }

  return {
    // Readable stores
    config: { subscribe: config.subscribe },
    loading: { subscribe: loading.subscribe },
    saving: { subscribe: saving.subscribe },
    error: { subscribe: error.subscribe },
    isDirty: { subscribe: isDirty.subscribe },
    validationErrors: { subscribe: validationErrors.subscribe },
    isValid: { subscribe: isValid.subscribe },

    // Actions
    load,
    save,
    reset,
    applyPreset,
    updateField,

    // Direct config update (for form bindings)
    set: config.set,
    update: config.update,
  };
}

// Validate configuration
function validateConfig(config: GeneratorConfig): ValidationError[] {
  const errors: ValidationError[] = [];

  // Global settings validation
  if (!config.global.start_date.match(/^\d{4}-\d{2}-\d{2}$/)) {
    errors.push({ field: 'global.start_date', message: 'Start date must be in YYYY-MM-DD format' });
  }

  if (config.global.period_months < 1 || config.global.period_months > 120) {
    errors.push({ field: 'global.period_months', message: 'Period must be between 1 and 120 months' });
  }

  if (config.global.memory_limit_mb < 0) {
    errors.push({ field: 'global.memory_limit_mb', message: 'Memory limit cannot be negative' });
  }

  // Company validation
  if (config.companies.length === 0) {
    errors.push({ field: 'companies', message: 'At least one company is required' });
  }

  config.companies.forEach((company, i) => {
    if (!company.code) {
      errors.push({ field: `companies[${i}].code`, message: `Company ${i + 1}: Code is required` });
    }
    if (!company.name) {
      errors.push({ field: `companies[${i}].name`, message: `Company ${i + 1}: Name is required` });
    }
    if (company.volume_weight <= 0) {
      errors.push({ field: `companies[${i}].volume_weight`, message: `Company ${i + 1}: Volume weight must be positive` });
    }
  });

  // Chart of accounts validation
  if (config.chart_of_accounts.min_hierarchy_depth < 1) {
    errors.push({ field: 'chart_of_accounts.min_hierarchy_depth', message: 'Minimum hierarchy depth must be at least 1' });
  }

  if (config.chart_of_accounts.max_hierarchy_depth < config.chart_of_accounts.min_hierarchy_depth) {
    errors.push({ field: 'chart_of_accounts.max_hierarchy_depth', message: 'Maximum hierarchy depth must be >= minimum' });
  }

  // Transaction settings validation
  if (config.transactions.amount_distribution.min_amount < 0) {
    errors.push({ field: 'transactions.amount_distribution.min_amount', message: 'Minimum amount cannot be negative' });
  }

  if (config.transactions.amount_distribution.max_amount <= config.transactions.amount_distribution.min_amount) {
    errors.push({ field: 'transactions.amount_distribution.max_amount', message: 'Maximum amount must be greater than minimum' });
  }

  // Fraud validation
  if (config.fraud.enabled && (config.fraud.fraud_rate < 0 || config.fraud.fraud_rate > 0.1)) {
    errors.push({ field: 'fraud.fraud_rate', message: 'Fraud rate must be between 0 and 10%' });
  }

  // Internal controls validation
  if (config.internal_controls.enabled) {
    if (config.internal_controls.exception_rate < 0 || config.internal_controls.exception_rate > 0.1) {
      errors.push({ field: 'internal_controls.exception_rate', message: 'Exception rate must be between 0 and 10%' });
    }
    if (config.internal_controls.sod_violation_rate < 0 || config.internal_controls.sod_violation_rate > 0.1) {
      errors.push({ field: 'internal_controls.sod_violation_rate', message: 'SoD violation rate must be between 0 and 10%' });
    }
  }

  return errors;
}

// Export the singleton store
export const configStore = createConfigStore();

// Industry options
export const INDUSTRIES = [
  { value: 'manufacturing', label: 'Manufacturing' },
  { value: 'retail', label: 'Retail' },
  { value: 'financial_services', label: 'Financial Services' },
  { value: 'healthcare', label: 'Healthcare' },
  { value: 'technology', label: 'Technology' },
  { value: 'professional_services', label: 'Professional Services' },
  { value: 'energy', label: 'Energy' },
  { value: 'transportation', label: 'Transportation' },
  { value: 'real_estate', label: 'Real Estate' },
  { value: 'telecommunications', label: 'Telecommunications' },
];

// CoA complexity options
export const COA_COMPLEXITIES = [
  { value: 'small', label: 'Small (~100 accounts)' },
  { value: 'medium', label: 'Medium (~400 accounts)' },
  { value: 'large', label: 'Large (~2500 accounts)' },
];

// Transaction volume options
export const TRANSACTION_VOLUMES = [
  { value: 'ten_k', label: '10K (Small)' },
  { value: 'hundred_k', label: '100K (Medium)' },
  { value: 'one_m', label: '1M (Large)' },
  { value: 'ten_m', label: '10M (Enterprise)' },
  { value: 'hundred_m', label: '100M (Massive)' },
];

// Output format options
export const OUTPUT_FORMATS = [
  { value: 'csv', label: 'CSV' },
  { value: 'json', label: 'JSON' },
  { value: 'parquet', label: 'Parquet (disabled)' },
];

// Compression options
export const COMPRESSION_OPTIONS = [
  { value: 'none', label: 'None' },
  { value: 'gzip', label: 'GZip' },
  { value: 'zstd', label: 'Zstandard' },
  { value: 'lz4', label: 'LZ4' },
];
