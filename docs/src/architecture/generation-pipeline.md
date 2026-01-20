# Generation Pipeline

Step-by-step generation process orchestrated by `datasynth-runtime`.

## Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                      GenerationOrchestrator                          │
│                                                                      │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│  │Init  │→│Master│→│Open  │→│Trans │→│Close │→│Inject│→│Export│   │
│  │      │ │Data  │ │Bal   │ │      │ │      │ │      │ │      │   │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Stage 1: Initialization

**Purpose:** Prepare generation environment

```rust
pub fn initialize(&mut self) -> Result<()> {
    // 1. Validate configuration
    ConfigValidator::new().validate(&self.config)?;

    // 2. Initialize RNG with seed
    self.rng = ChaCha8Rng::seed_from_u64(self.config.global.seed);

    // 3. Create UUID factory
    self.uuid_factory = DeterministicUuidFactory::new(self.config.global.seed);

    // 4. Set up memory guard
    self.memory_guard = MemoryGuard::new(self.config.memory_config());

    // 5. Create output directories
    fs::create_dir_all(&self.output_path)?;

    Ok(())
}
```

**Outputs:**
- Validated configuration
- Initialized RNG
- UUID factory
- Memory guard
- Output directories

## Stage 2: Master Data Generation

**Purpose:** Generate all entity master records

```rust
pub fn generate_master_data(&mut self) -> Result<MasterDataState> {
    let mut state = MasterDataState::new();

    // 1. Chart of Accounts
    let coa_gen = CoaGenerator::new(&self.config, &mut self.rng);
    state.chart_of_accounts = coa_gen.generate()?;

    // 2. Employees (needed for approvals)
    let emp_gen = EmployeeGenerator::new(&self.config, &mut self.rng);
    state.employees = emp_gen.generate()?;

    // 3. Vendors (reference employees)
    let vendor_gen = VendorGenerator::new(&self.config, &mut self.rng);
    state.vendors = vendor_gen.generate()?;

    // 4. Customers
    let customer_gen = CustomerGenerator::new(&self.config, &mut self.rng);
    state.customers = customer_gen.generate()?;

    // 5. Materials
    let material_gen = MaterialGenerator::new(&self.config, &mut self.rng);
    state.materials = material_gen.generate()?;

    // 6. Fixed Assets
    let asset_gen = AssetGenerator::new(&self.config, &mut self.rng);
    state.fixed_assets = asset_gen.generate()?;

    // 7. Register in entity registry
    self.registry.register_all(&state);

    Ok(state)
}
```

**Outputs:**
- Chart of Accounts
- Vendors, Customers
- Materials, Fixed Assets
- Employees
- Entity Registry

## Stage 3: Opening Balance Generation

**Purpose:** Create coherent opening balance sheet

```rust
pub fn generate_opening_balances(&mut self) -> Result<Vec<JournalEntry>> {
    let generator = OpeningBalanceGenerator::new(
        &self.config,
        &self.state.chart_of_accounts,
        &mut self.rng,
    );

    let entries = generator.generate()?;

    // Initialize balance tracker
    self.balance_tracker = BalanceTracker::new(&self.state.chart_of_accounts);
    for entry in &entries {
        self.balance_tracker.post(entry)?;
    }

    // Verify A = L + E
    assert!(self.balance_tracker.is_balanced());

    Ok(entries)
}
```

**Outputs:**
- Opening balance entries
- Initialized balance tracker

## Stage 4: Transaction Generation

**Purpose:** Generate main transaction volume

```rust
pub fn generate_transactions(&mut self) -> Result<Vec<JournalEntry>> {
    let target = self.config.transactions.target_count;
    let mut entries = Vec::with_capacity(target as usize);

    // Calculate counts by source
    let p2p_count = (target as f64 * self.config.document_flows.p2p.flow_rate) as u64;
    let o2c_count = (target as f64 * self.config.document_flows.o2c.flow_rate) as u64;
    let other_count = target - p2p_count - o2c_count;

    // 1. Generate P2P flows
    let p2p_entries = self.generate_p2p_flows(p2p_count)?;
    entries.extend(p2p_entries);

    // 2. Generate O2C flows
    let o2c_entries = self.generate_o2c_flows(o2c_count)?;
    entries.extend(o2c_entries);

    // 3. Generate other entries (manual, recurring, etc.)
    let other_entries = self.generate_other_entries(other_count)?;
    entries.extend(other_entries);

    // 4. Sort by posting date
    entries.sort_by_key(|e| e.header.posting_date);

    // 5. Update balance tracker
    for entry in &entries {
        self.balance_tracker.post(entry)?;
    }

    Ok(entries)
}
```

### P2P Flow Generation

```rust
fn generate_p2p_flows(&mut self, count: u64) -> Result<Vec<JournalEntry>> {
    let mut p2p_gen = P2pGenerator::new(&self.config, &self.registry, &mut self.rng);
    let mut doc_gen = DocumentFlowJeGenerator::new(&self.config);

    let mut entries = Vec::new();

    for _ in 0..count {
        // 1. Generate document flow
        let flow = p2p_gen.generate_flow()?;
        self.state.documents.add_p2p_flow(&flow);

        // 2. Generate journal entries from flow
        let flow_entries = doc_gen.generate_from_p2p(&flow)?;
        entries.extend(flow_entries);
    }

    Ok(entries)
}
```

**Outputs:**
- Journal entries
- Document records
- Updated balances

## Stage 5: Period Close

**Purpose:** Run period-end processes

```rust
pub fn run_period_close(&mut self) -> Result<()> {
    let close_engine = CloseEngine::new(&self.config.period_close);

    for period in self.config.periods() {
        // 1. Monthly close
        let monthly_entries = close_engine.run_monthly_close(
            period,
            &self.state,
            &mut self.balance_tracker,
        )?;
        self.state.entries.extend(monthly_entries);

        // 2. Quarterly close (if applicable)
        if period.is_quarter_end() {
            let quarterly_entries = close_engine.run_quarterly_close(
                period,
                &self.state,
            )?;
            self.state.entries.extend(quarterly_entries);
        }

        // 3. Generate trial balance
        let trial_balance = self.balance_tracker.to_trial_balance(period);
        self.state.trial_balances.push(trial_balance);
    }

    // 4. Annual close
    if self.config.has_year_end() {
        let annual_entries = close_engine.run_annual_close(&self.state)?;
        self.state.entries.extend(annual_entries);
    }

    Ok(())
}
```

**Outputs:**
- Accrual entries
- Depreciation entries
- Closing entries
- Trial balances

## Stage 6: Anomaly Injection

**Purpose:** Add anomalies and generate labels

```rust
pub fn inject_anomalies(&mut self) -> Result<()> {
    if !self.config.anomaly_injection.enabled {
        return Ok(());
    }

    let mut injector = AnomalyInjector::new(
        &self.config.anomaly_injection,
        &mut self.rng,
    );

    // 1. Select entries for injection
    let target_count = (self.state.entries.len() as f64
        * self.config.anomaly_injection.total_rate) as usize;

    // 2. Inject anomalies
    let (modified, labels) = injector.inject(
        &mut self.state.entries,
        target_count,
    )?;

    // 3. Store labels
    self.state.anomaly_labels = labels;

    // 4. Apply data quality variations
    if self.config.data_quality.enabled {
        let dq_injector = DataQualityInjector::new(&self.config.data_quality);
        dq_injector.apply(&mut self.state)?;
    }

    Ok(())
}
```

**Outputs:**
- Modified entries with anomalies
- Anomaly labels for ML

## Stage 7: Export

**Purpose:** Write all outputs

```rust
pub fn export(&self) -> Result<()> {
    // 1. Master data
    self.export_master_data()?;

    // 2. Transactions
    self.export_transactions()?;

    // 3. Documents
    self.export_documents()?;

    // 4. Subledgers
    self.export_subledgers()?;

    // 5. Trial balances
    self.export_trial_balances()?;

    // 6. Labels
    self.export_labels()?;

    // 7. Controls
    self.export_controls()?;

    // 8. Graphs (if enabled)
    if self.config.graph_export.enabled {
        self.export_graphs()?;
    }

    Ok(())
}
```

**Outputs:**
- CSV/JSON files
- Graph exports
- Label files

## Parallel Execution

Stages that support parallelism:

```rust
// Parallel transaction generation
let entries: Vec<JournalEntry> = (0..num_threads)
    .into_par_iter()
    .flat_map(|thread_id| {
        let mut gen = JournalEntryGenerator::new(
            &config,
            seed + thread_id as u64,
        );
        gen.generate_batch(batch_size)
    })
    .collect();
```

## Progress Tracking

```rust
pub fn run_with_progress<F>(&mut self, callback: F) -> Result<()>
where
    F: Fn(Progress),
{
    let tracker = ProgressTracker::new(self.config.total_items());

    for stage in self.stages() {
        tracker.set_phase(&stage.name);
        stage.run()?;
        tracker.advance(stage.items);
        callback(tracker.progress());
    }

    Ok(())
}
```

## See Also

- [Data Flow](data-flow.md)
- [datasynth-runtime](../crates/datasynth-runtime.md)
- [Memory Management](memory-management.md)
