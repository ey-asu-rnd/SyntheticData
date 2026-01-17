<script lang="ts">
  import { configStore, INDUSTRIES, createDefaultConfig, type GeneratorConfig } from '$lib/stores/config';
  import { FormSection } from '$lib/components/forms';

  const config = configStore.config;
  const isDirty = configStore.isDirty;

  // Deep partial type for industry overrides
  type DeepPartial<T> = {
    [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
  };

  // Comprehensive industry preset configurations
  function createPreset(industry: string, overrides: DeepPartial<GeneratorConfig>): GeneratorConfig {
    const base = createDefaultConfig();
    base.global.industry = industry;

    // Deep merge overrides
    if (overrides.global) Object.assign(base.global, overrides.global);
    if (overrides.master_data) Object.assign(base.master_data, overrides.master_data);
    if (overrides.transactions) {
      if (overrides.transactions.line_item_distribution) Object.assign(base.transactions.line_item_distribution, overrides.transactions.line_item_distribution);
      if (overrides.transactions.amount_distribution) Object.assign(base.transactions.amount_distribution, overrides.transactions.amount_distribution);
      if (overrides.transactions.source_distribution) Object.assign(base.transactions.source_distribution, overrides.transactions.source_distribution);
      if (overrides.transactions.seasonality) Object.assign(base.transactions.seasonality, overrides.transactions.seasonality);
    }
    if (overrides.document_flows) {
      if (overrides.document_flows.p2p) Object.assign(base.document_flows.p2p, overrides.document_flows.p2p);
      if (overrides.document_flows.o2c) Object.assign(base.document_flows.o2c, overrides.document_flows.o2c);
    }
    if (overrides.fraud) Object.assign(base.fraud, overrides.fraud);
    if (overrides.internal_controls) Object.assign(base.internal_controls, overrides.internal_controls);

    return base;
  }

  // Industry preset overrides (merged with defaults via createPreset)
  const industryPresets: Record<string, DeepPartial<GeneratorConfig>> = {
    manufacturing: {
      global: { industry: 'manufacturing', period_months: 12 },
      master_data: {
        vendors: { count: 200, distribution: { payment_terms_net30: 0.35, payment_terms_net60: 0.35, payment_terms_net90: 0.20, payment_terms_immediate: 0.10 } },
        customers: { count: 150, distribution: { credit_rating_aaa: 0.10, credit_rating_aa: 0.25, credit_rating_a: 0.35, credit_rating_bbb: 0.20, credit_rating_bb: 0.10 } },
        materials: { count: 500, distribution: { raw_material: 0.35, semi_finished: 0.25, finished_goods: 0.30, trading_goods: 0.05, services: 0.05 } },
        assets: { count: 100, distribution: { buildings: 0.15, machinery: 0.40, vehicles: 0.15, furniture: 0.10, it_equipment: 0.15, intangibles: 0.05 } },
        employees: { count: 50, distribution: { finance: 0.20, operations: 0.40, sales: 0.15, procurement: 0.20, management: 0.05 } },
      },
      transactions: {
        line_item_distribution: { '2': 0.50, '3': 0.10, '4': 0.20, '5': 0.05, '6': 0.05, '7-9': 0.05, '10-99': 0.05 },
        amount_distribution: { min_amount: 0.01, max_amount: 50000000, lognormal_mu: 8.0, lognormal_sigma: 2.2, round_number_probability: 0.20, nice_number_probability: 0.10, benford_compliance: true },
        source_distribution: { manual: 0.05, interface: 0.35, batch: 0.45, recurring: 0.15 },
        seasonality: { month_end_spike: true, month_end_multiplier: 2.0, quarter_end_spike: true, quarter_end_multiplier: 3.5, year_end_spike: true, year_end_multiplier: 5.0, day_of_week_patterns: true },
      },
      document_flows: {
        p2p: { enabled: true, three_way_match_rate: 0.92, partial_delivery_rate: 0.20, price_variance_rate: 0.10, average_po_to_gr_days: 21, average_gr_to_invoice_days: 7, average_invoice_to_payment_days: 45 },
        o2c: { enabled: true, credit_check_failure_rate: 0.03, partial_shipment_rate: 0.15, return_rate: 0.05, bad_debt_rate: 0.02 },
      },
      fraud: { enabled: false, fraud_rate: 0.003 },
      internal_controls: { enabled: true, exception_rate: 0.02, sod_violation_rate: 0.01 },
    },
    retail: {
      global: { industry: 'retail', period_months: 12 },
      master_data: {
        vendors: { count: 100, distribution: { payment_terms_net30: 0.50, payment_terms_net60: 0.30, payment_terms_net90: 0.10, payment_terms_immediate: 0.10 } },
        customers: { count: 5000, distribution: { credit_rating_aaa: 0.05, credit_rating_aa: 0.15, credit_rating_a: 0.30, credit_rating_bbb: 0.30, credit_rating_bb: 0.20 } },
        materials: { count: 2000, distribution: { raw_material: 0.05, semi_finished: 0.05, finished_goods: 0.10, trading_goods: 0.75, services: 0.05 } },
        assets: { count: 50, distribution: { buildings: 0.25, machinery: 0.10, vehicles: 0.15, furniture: 0.25, it_equipment: 0.20, intangibles: 0.05 } },
        employees: { count: 100, distribution: { finance: 0.15, operations: 0.35, sales: 0.35, procurement: 0.10, management: 0.05 } },
      },
      transactions: {
        line_item_distribution: { '2': 0.70, '3': 0.10, '4': 0.10, '5': 0.03, '6': 0.03, '7-9': 0.02, '10-99': 0.02 },
        amount_distribution: { min_amount: 0.01, max_amount: 10000000, lognormal_mu: 5.5, lognormal_sigma: 2.8, round_number_probability: 0.35, nice_number_probability: 0.25, benford_compliance: true },
        source_distribution: { manual: 0.05, interface: 0.50, batch: 0.35, recurring: 0.10 },
        seasonality: { month_end_spike: true, month_end_multiplier: 1.5, quarter_end_spike: true, quarter_end_multiplier: 2.5, year_end_spike: true, year_end_multiplier: 8.0, day_of_week_patterns: true },
      },
      document_flows: {
        p2p: { enabled: true, three_way_match_rate: 0.85, partial_delivery_rate: 0.25, price_variance_rate: 0.15, average_po_to_gr_days: 7, average_gr_to_invoice_days: 3, average_invoice_to_payment_days: 30 },
        o2c: { enabled: true, credit_check_failure_rate: 0.05, partial_shipment_rate: 0.08, return_rate: 0.08, bad_debt_rate: 0.03 },
      },
      fraud: { enabled: false, fraud_rate: 0.004 },
      internal_controls: { enabled: true, exception_rate: 0.03, sod_violation_rate: 0.015 },
    },
    financial_services: {
      global: { industry: 'financial_services', period_months: 12 },
      master_data: {
        vendors: { count: 50, distribution: { payment_terms_net30: 0.60, payment_terms_net60: 0.25, payment_terms_net90: 0.10, payment_terms_immediate: 0.05 } },
        customers: { count: 1000, distribution: { credit_rating_aaa: 0.15, credit_rating_aa: 0.30, credit_rating_a: 0.35, credit_rating_bbb: 0.15, credit_rating_bb: 0.05 } },
        materials: { count: 20, distribution: { raw_material: 0.00, semi_finished: 0.00, finished_goods: 0.00, trading_goods: 0.10, services: 0.90 } },
        assets: { count: 200, distribution: { buildings: 0.30, machinery: 0.05, vehicles: 0.05, furniture: 0.15, it_equipment: 0.30, intangibles: 0.15 } },
        employees: { count: 200, distribution: { finance: 0.50, operations: 0.15, sales: 0.20, procurement: 0.05, management: 0.10 } },
      },
      transactions: {
        line_item_distribution: { '2': 0.55, '3': 0.15, '4': 0.15, '5': 0.05, '6': 0.05, '7-9': 0.03, '10-99': 0.02 },
        amount_distribution: { min_amount: 0.01, max_amount: 500000000, lognormal_mu: 10.0, lognormal_sigma: 3.0, round_number_probability: 0.15, nice_number_probability: 0.05, benford_compliance: true },
        source_distribution: { manual: 0.03, interface: 0.60, batch: 0.30, recurring: 0.07 },
        seasonality: { month_end_spike: true, month_end_multiplier: 3.0, quarter_end_spike: true, quarter_end_multiplier: 5.0, year_end_spike: true, year_end_multiplier: 7.0, day_of_week_patterns: true },
      },
      document_flows: {
        p2p: { enabled: true, three_way_match_rate: 0.98, partial_delivery_rate: 0.05, price_variance_rate: 0.02, average_po_to_gr_days: 3, average_gr_to_invoice_days: 2, average_invoice_to_payment_days: 15 },
        o2c: { enabled: true, credit_check_failure_rate: 0.01, partial_shipment_rate: 0.02, return_rate: 0.01, bad_debt_rate: 0.005 },
      },
      fraud: { enabled: false, fraud_rate: 0.002 },
      internal_controls: { enabled: true, exception_rate: 0.01, sod_violation_rate: 0.005, sox_materiality_threshold: 50000 },
    },
    healthcare: {
      global: { industry: 'healthcare', period_months: 12 },
      master_data: {
        vendors: { count: 300, distribution: { payment_terms_net30: 0.45, payment_terms_net60: 0.35, payment_terms_net90: 0.15, payment_terms_immediate: 0.05 } },
        customers: { count: 200, distribution: { credit_rating_aaa: 0.20, credit_rating_aa: 0.30, credit_rating_a: 0.30, credit_rating_bbb: 0.15, credit_rating_bb: 0.05 } },
        materials: { count: 1000, distribution: { raw_material: 0.20, semi_finished: 0.15, finished_goods: 0.25, trading_goods: 0.30, services: 0.10 } },
        assets: { count: 500, distribution: { buildings: 0.25, machinery: 0.35, vehicles: 0.10, furniture: 0.10, it_equipment: 0.15, intangibles: 0.05 } },
        employees: { count: 150, distribution: { finance: 0.20, operations: 0.40, sales: 0.10, procurement: 0.20, management: 0.10 } },
      },
      transactions: {
        line_item_distribution: { '2': 0.45, '3': 0.15, '4': 0.20, '5': 0.08, '6': 0.05, '7-9': 0.04, '10-99': 0.03 },
        amount_distribution: { min_amount: 0.01, max_amount: 100000000, lognormal_mu: 8.5, lognormal_sigma: 2.5, round_number_probability: 0.20, nice_number_probability: 0.10, benford_compliance: true },
        source_distribution: { manual: 0.08, interface: 0.45, batch: 0.35, recurring: 0.12 },
        seasonality: { month_end_spike: true, month_end_multiplier: 2.5, quarter_end_spike: true, quarter_end_multiplier: 4.0, year_end_spike: true, year_end_multiplier: 5.5, day_of_week_patterns: true },
      },
      document_flows: {
        p2p: { enabled: true, three_way_match_rate: 0.90, partial_delivery_rate: 0.12, price_variance_rate: 0.08, average_po_to_gr_days: 10, average_gr_to_invoice_days: 5, average_invoice_to_payment_days: 45 },
        o2c: { enabled: true, credit_check_failure_rate: 0.02, partial_shipment_rate: 0.10, return_rate: 0.03, bad_debt_rate: 0.01 },
      },
      fraud: { enabled: false, fraud_rate: 0.002 },
      internal_controls: { enabled: true, exception_rate: 0.015, sod_violation_rate: 0.008 },
    },
    technology: {
      global: { industry: 'technology', period_months: 12 },
      master_data: {
        vendors: { count: 100, distribution: { payment_terms_net30: 0.55, payment_terms_net60: 0.30, payment_terms_net90: 0.10, payment_terms_immediate: 0.05 } },
        customers: { count: 500, distribution: { credit_rating_aaa: 0.15, credit_rating_aa: 0.25, credit_rating_a: 0.35, credit_rating_bbb: 0.20, credit_rating_bb: 0.05 } },
        materials: { count: 50, distribution: { raw_material: 0.05, semi_finished: 0.05, finished_goods: 0.15, trading_goods: 0.25, services: 0.50 } },
        assets: { count: 300, distribution: { buildings: 0.10, machinery: 0.05, vehicles: 0.05, furniture: 0.15, it_equipment: 0.45, intangibles: 0.20 } },
        employees: { count: 100, distribution: { finance: 0.15, operations: 0.30, sales: 0.25, procurement: 0.10, management: 0.20 } },
      },
      transactions: {
        line_item_distribution: { '2': 0.60, '3': 0.12, '4': 0.12, '5': 0.06, '6': 0.04, '7-9': 0.04, '10-99': 0.02 },
        amount_distribution: { min_amount: 0.01, max_amount: 50000000, lognormal_mu: 7.5, lognormal_sigma: 2.8, round_number_probability: 0.25, nice_number_probability: 0.15, benford_compliance: true },
        source_distribution: { manual: 0.05, interface: 0.55, batch: 0.25, recurring: 0.15 },
        seasonality: { month_end_spike: true, month_end_multiplier: 2.0, quarter_end_spike: true, quarter_end_multiplier: 3.5, year_end_spike: true, year_end_multiplier: 6.0, day_of_week_patterns: true },
      },
      document_flows: {
        p2p: { enabled: true, three_way_match_rate: 0.88, partial_delivery_rate: 0.08, price_variance_rate: 0.05, average_po_to_gr_days: 5, average_gr_to_invoice_days: 3, average_invoice_to_payment_days: 30 },
        o2c: { enabled: true, credit_check_failure_rate: 0.02, partial_shipment_rate: 0.05, return_rate: 0.04, bad_debt_rate: 0.015 },
      },
      fraud: { enabled: false, fraud_rate: 0.003 },
      internal_controls: { enabled: true, exception_rate: 0.02, sod_violation_rate: 0.01 },
    },
  };

  // Preset descriptions
  const presetDescriptions: Record<string, { title: string; description: string; icon: string; characteristics: string[] }> = {
    manufacturing: {
      title: 'Manufacturing',
      description: 'Heavy inventory, bill of materials, production processes',
      icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 0 0-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 0 0-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 0 0-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 0 0-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 0 0 1.066-2.573c-.94-1.543.826-3.31 2.37-2.37 1 .608 2.296.07 2.572-1.065z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z',
      characteristics: ['High vendor count', 'Complex BOM structures', 'Inventory movements', 'Production orders', 'Quality management'],
    },
    retail: {
      title: 'Retail',
      description: 'High customer volume, point of sale, inventory turnover',
      icon: 'M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 1 0 0 4 2 2 0 0 0 0-4zm-8 2a2 2 0 1 1-4 0 2 2 0 0 1 4 0z',
      characteristics: ['Very high customer count', 'High SKU count', 'Fast inventory turns', 'Cash transactions', 'Seasonal patterns'],
    },
    financial_services: {
      title: 'Financial Services',
      description: 'Complex transactions, regulatory compliance, high value',
      icon: 'M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z',
      characteristics: ['Low material count', 'High employee count', 'Large fixed assets', 'Complex intercompany', 'Strict controls'],
    },
    healthcare: {
      title: 'Healthcare',
      description: 'Medical supplies, equipment, regulatory requirements',
      icon: 'M4.318 6.318a4.5 4.5 0 0 0 0 6.364L12 20.364l7.682-7.682a4.5 4.5 0 0 0-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 0 0-6.364 0z',
      characteristics: ['High vendor diversity', 'Expensive equipment', 'Batch tracking', 'Compliance heavy', 'Service revenue'],
    },
    technology: {
      title: 'Technology',
      description: 'Software, services, R&D, subscription revenue',
      icon: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2z',
      characteristics: ['Low inventory', 'High IT assets', 'Subscription billing', 'R&D capitalization', 'Stock compensation'],
    },
  };

  // Apply a preset
  function applyPreset(industry: string) {
    const preset = industryPresets[industry];
    if (preset && $config) {
      const newConfig = createPreset(industry, preset);
      configStore.applyPreset(newConfig);
    }
  }

  // Export configuration as JSON
  function exportConfig() {
    if (!$config) return;
    const dataStr = JSON.stringify($config, null, 2);
    const blob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `synth-config-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  // Import configuration from JSON
  let fileInput: HTMLInputElement;

  function importConfig() {
    fileInput?.click();
  }

  function handleFileSelect(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const imported = JSON.parse(e.target?.result as string) as GeneratorConfig;
        configStore.applyPreset(imported);
      } catch (err) {
        console.error('Failed to parse config file:', err);
        alert('Failed to parse configuration file. Please ensure it is valid JSON.');
      }
    };
    reader.readAsText(file);
    // Reset input so same file can be selected again
    target.value = '';
  }

  // Reset to defaults
  function resetToDefaults() {
    if (confirm('Reset all settings to defaults? This cannot be undone.')) {
      configStore.applyPreset(createDefaultConfig());
    }
  }
</script>

<div class="page">
  <header class="page-header">
    <div>
      <h1>Presets & Configuration</h1>
      <p>Apply industry presets or import/export configuration files</p>
    </div>
  </header>

  <div class="sections">
    <!-- Industry Presets -->
    <FormSection
      title="Industry Presets"
      description="Quick-start configurations optimized for specific industries"
    >
      <div class="section-content">
        <p class="section-intro">
          Select an industry preset to configure the generator with typical settings for that sector.
          Presets adjust master data counts, transaction patterns, and industry-specific behaviors.
        </p>

        <div class="preset-grid">
          {#each Object.entries(presetDescriptions) as [key, preset]}
            {@const isActive = $config?.global.industry === key}
            <button
              type="button"
              class="preset-card"
              class:active={isActive}
              onclick={() => applyPreset(key)}
            >
              <div class="preset-header">
                <span class="preset-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d={preset.icon} />
                  </svg>
                </span>
                <div class="preset-title-group">
                  <span class="preset-title">{preset.title}</span>
                  <span class="preset-description">{preset.description}</span>
                </div>
                {#if isActive}
                  <span class="active-badge">Active</span>
                {/if}
              </div>
              <ul class="preset-characteristics">
                {#each preset.characteristics as char}
                  <li>{char}</li>
                {/each}
              </ul>
            </button>
          {/each}
        </div>
      </div>
    </FormSection>

    <!-- Import/Export -->
    <FormSection
      title="Import / Export Configuration"
      description="Save your configuration or load a previously saved one"
    >
      <div class="section-content">
        <div class="action-cards">
          <div class="action-card">
            <div class="action-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12" />
              </svg>
            </div>
            <div class="action-content">
              <h4>Export Configuration</h4>
              <p>Download current configuration as a JSON file for backup or sharing.</p>
              <button type="button" class="action-btn" onclick={exportConfig}>
                Export JSON
              </button>
            </div>
          </div>

          <div class="action-card">
            <div class="action-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
              </svg>
            </div>
            <div class="action-content">
              <h4>Import Configuration</h4>
              <p>Load a previously exported configuration file.</p>
              <button type="button" class="action-btn" onclick={importConfig}>
                Import JSON
              </button>
              <input
                bind:this={fileInput}
                type="file"
                accept=".json"
                onchange={handleFileSelect}
                style="display: none;"
              />
            </div>
          </div>

          <div class="action-card">
            <div class="action-icon warning">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 0 0 4.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 0 1-15.357-2m15.357 2H15" />
              </svg>
            </div>
            <div class="action-content">
              <h4>Reset to Defaults</h4>
              <p>Clear all customizations and restore factory settings.</p>
              <button type="button" class="action-btn danger" onclick={resetToDefaults}>
                Reset All
              </button>
            </div>
          </div>
        </div>
      </div>
    </FormSection>

    <!-- Quick Stats -->
    {#if $config}
      <FormSection
        title="Current Configuration Summary"
        description="Overview of your current settings"
      >
        <div class="section-content">
          <div class="stats-grid">
            <div class="stat-card">
              <span class="stat-label">Industry</span>
              <span class="stat-value">{INDUSTRIES.find(i => i.value === $config.global.industry)?.label || $config.global.industry}</span>
            </div>
            <div class="stat-card">
              <span class="stat-label">Companies</span>
              <span class="stat-value">{$config.companies.length}</span>
            </div>
            <div class="stat-card">
              <span class="stat-label">Period</span>
              <span class="stat-value">{$config.global.period_months} months</span>
            </div>
            <div class="stat-card">
              <span class="stat-label">Vendors</span>
              <span class="stat-value">{$config.master_data.vendors.count}</span>
            </div>
            <div class="stat-card">
              <span class="stat-label">Customers</span>
              <span class="stat-value">{$config.master_data.customers.count}</span>
            </div>
            <div class="stat-card">
              <span class="stat-label">Materials</span>
              <span class="stat-value">{$config.master_data.materials.count}</span>
            </div>
            <div class="stat-card">
              <span class="stat-label">Assets</span>
              <span class="stat-value">{$config.master_data.assets.count}</span>
            </div>
            <div class="stat-card">
              <span class="stat-label">Employees</span>
              <span class="stat-value">{$config.master_data.employees.count}</span>
            </div>
          </div>

          <div class="feature-flags">
            <span class="flag" class:enabled={$config.document_flows.p2p.enabled}>P2P Flow</span>
            <span class="flag" class:enabled={$config.document_flows.o2c.enabled}>O2C Flow</span>
            <span class="flag" class:enabled={$config.fraud.enabled}>Fraud Simulation</span>
            <span class="flag" class:enabled={$config.internal_controls.enabled}>Internal Controls</span>
            <span class="flag" class:enabled={$config.balance.generate_opening_balance}>Opening Balance</span>
          </div>

          {#if $isDirty}
            <div class="unsaved-warning">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <span>You have unsaved changes. Go to a config page and save to persist.</span>
            </div>
          {/if}
        </div>
      </FormSection>
    {/if}
  </div>
</div>

<style>
  .page {
    max-width: 1000px;
  }

  .page-header {
    margin-bottom: var(--space-6);
  }

  .page-header h1 {
    margin-bottom: var(--space-1);
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }

  .section-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .section-intro {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.5;
  }

  .preset-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: var(--space-4);
  }

  .preset-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    background-color: var(--color-background);
    border: 2px solid var(--color-border);
    border-radius: var(--radius-lg);
    text-align: left;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .preset-card:hover {
    border-color: var(--color-accent);
  }

  .preset-card.active {
    border-color: var(--color-accent);
    background-color: color-mix(in srgb, var(--color-accent) 5%, var(--color-background));
  }

  .preset-header {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
  }

  .preset-icon {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-surface);
    border-radius: var(--radius-md);
    color: var(--color-accent);
    flex-shrink: 0;
  }

  .preset-icon svg {
    width: 22px;
    height: 22px;
  }

  .preset-title-group {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .preset-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .preset-description {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .active-badge {
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: var(--space-1) var(--space-2);
    background-color: var(--color-accent);
    color: white;
    border-radius: var(--radius-sm);
  }

  .preset-characteristics {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .preset-characteristics li {
    font-size: 0.6875rem;
    color: var(--color-text-muted);
    padding: 2px var(--space-2);
    background-color: var(--color-surface);
    border-radius: var(--radius-sm);
  }

  .action-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--space-4);
  }

  .action-card {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-4);
    background-color: var(--color-background);
    border-radius: var(--radius-lg);
  }

  .action-icon {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-surface);
    border-radius: var(--radius-md);
    color: var(--color-accent);
    flex-shrink: 0;
  }

  .action-icon.warning {
    color: var(--color-warning);
  }

  .action-icon svg {
    width: 24px;
    height: 24px;
  }

  .action-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .action-content h4 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .action-content p {
    font-size: 0.8125rem;
    color: var(--color-text-secondary);
    margin: 0;
    line-height: 1.4;
  }

  .action-btn {
    align-self: flex-start;
    padding: var(--space-2) var(--space-3);
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--color-accent);
    background-color: transparent;
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .action-btn:hover {
    background-color: var(--color-accent);
    color: white;
  }

  .action-btn.danger {
    color: var(--color-danger);
    border-color: var(--color-danger);
  }

  .action-btn.danger:hover {
    background-color: var(--color-danger);
    color: white;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: var(--space-3);
  }

  .stat-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    padding: var(--space-3);
    background-color: var(--color-background);
    border-radius: var(--radius-md);
    text-align: center;
  }

  .stat-label {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .stat-value {
    font-family: var(--font-mono);
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .feature-flags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .flag {
    font-size: 0.75rem;
    font-weight: 500;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    background-color: var(--color-background);
    color: var(--color-text-muted);
  }

  .flag.enabled {
    background-color: color-mix(in srgb, var(--color-success) 15%, transparent);
    color: var(--color-success);
  }

  .unsaved-warning {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3);
    background-color: color-mix(in srgb, var(--color-warning) 10%, transparent);
    border: 1px solid var(--color-warning);
    border-radius: var(--radius-md);
    font-size: 0.8125rem;
    color: var(--color-warning);
  }

  .unsaved-warning svg {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
  }
</style>
