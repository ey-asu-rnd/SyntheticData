# Roadmap: Enterprise Simulation & ML Ground Truth

This roadmap outlines future enhancements, new features, and developer experience (DX) improvements. It is organized by themes and phased execution, with options to prioritize based on customer or research needs.

## Completed Features (v0.1.0 - v0.2.0)

The following major capabilities have been implemented:

### Core Data Generation
- Statistical distributions based on empirical GL research
- Benford's Law compliance with configurable fraud patterns
- Industry presets (manufacturing, retail, financial services, healthcare, technology)
- Chart of Accounts complexity levels (small, medium, large)
- Temporal patterns and regional holiday calendars

### Enterprise Simulation
- Master data management (vendors, customers, materials, assets, employees)
- Document flow engine (P2P, O2C with three-way matching)
- Intercompany transactions with IC matching and transfer pricing
- Balance coherence (opening balances, running balance tracking, trial balance)
- Subledger simulation (AR, AP, Fixed Assets, Inventory)
- Currency and FX (realistic exchange rates, translation, CTA)
- Period close engine (monthly close, depreciation, accruals, year-end)

### ML & Analytics
- Graph export (PyTorch Geometric, Neo4j, DGL)
- Anomaly injection (20+ fraud types with labeling)
- Data quality variations (missing values, typos, duplicates)
- Evaluation framework with auto-tuning

### Domain-Specific Modules
- Banking/KYC/AML with customer personas and fraud typologies
- OCEL 2.0 process mining event logs
- Audit simulation (ISA-compliant engagements, workpapers, findings)

### Production Features
- REST/gRPC/WebSocket server with auth and rate limiting
- Desktop UI (Tauri/SvelteKit)
- Resource guards (memory, disk, CPU) with graceful degradation
- Python wrapper with blueprints

### Privacy & Compliance (v0.2.0)
- Privacy-preserving fingerprint extraction
- Differential privacy (Laplace/Gaussian mechanisms)
- K-anonymity for categorical values
- Fidelity evaluation for synthetic data

---

## Guiding Goals

- **Enterprise realism**: Simulate multi-entity, multi-region, and multi-system operations with coherent process flows
- **ML ground truth**: Capture true labels and causal factors for supervised learning, explainability, and evaluation
- **Scalability**: Handle large volumes with stable performance and repeatable results
- **DX productivity**: Accelerate experimentation, onboarding, and customization

---

## Future Opportunity Areas

### 1) Enhanced Enterprise Process Fidelity

**Planned Enhancements**
- Record-to-Report (R2R) and Hire-to-Retire (H2R) process flows
- Event-driven transaction chains with SLA constraints
- Operational disruptions (supply shocks, outages) for realistic anomalies

**Feature Options**
- **Scenario packs** for additional industries
- **Policy engines** for configurable approvals and delegation of authority
- **System migration patterns** simulating ERP transitions

### 2) Advanced Ground Truth & Labeling

**Planned Enhancements**
- Ground truth provenance graph linking all generated artifacts
- Causal metadata for "why" labels exist
- Counterfactual generation for sensitivity testing

**Feature Options**
- **Label confidence scoring** for semi-supervised workflows
- **Bias controls** for intentional class imbalance
- **Explanation generation** for anomaly labels

### 3) System & Integration Realism

**Planned Enhancements**
- Multi-system source simulation (ERP + CRM + WMS)
- Reconciliation gap generation between systems
- System performance metadata (latency, retries)

**Feature Options**
- **Connector emulator** output (SAP, NetSuite, Dynamics-style exports)
- **CDC/event stream** exports with offsets and watermarking
- **API response mocking** for integration testing

### 4) Advanced Observability & Evaluation

**Planned Enhancements**
- Run-level metadata snapshots for reproducibility
- Extended benchmark suites for specific domains
- Distribution drift monitoring

**Feature Options**
- **Interactive dashboards** for generation monitoring
- **Automated regression testing** against golden datasets
- **Leaderboards** for benchmark comparison

### 5) Developer Experience (DX)

**Planned Enhancements**
- Template-driven project scaffolding
- Plugin SDK for custom generators
- Interactive scenario builder

**Feature Options**
- **Visual configuration editor** for non-technical users
- **Replayable runs** with caching and partial regeneration
- **Schema-aware IDE extensions** for config authoring

---

## Phased Roadmap

### Phase: Near-Term (Next Release)

**Objectives**
- Extend fingerprinting capabilities
- Improve Python wrapper ecosystem
- Add additional industry presets

**Candidate Deliverables**
- Fingerprint-to-config synthesis improvements
- Additional Python blueprints
- Energy/utilities industry preset
- Enhanced fidelity metrics

### Phase: Medium-Term

**Objectives**
- Multi-system integration simulation
- Enhanced ground truth lineage

**Candidate Deliverables**
- R2R and H2R process flows
- Multi-source reconciliation gaps
- Provenance graph for all generated data
- Plugin SDK documentation

### Phase: Long-Term

**Objectives**
- Advanced ML research features
- Enterprise-scale testing capabilities

**Candidate Deliverables**
- Counterfactual generation
- Operational disruption modeling
- Standardized benchmark suites
- Interactive scenario builder

---

## Dependencies & Risks

- **Schema stability**: Ensure backward compatibility as new features are added
- **Performance**: Larger simulations may require additional optimization
- **Validation**: Avoid unrealistic combinations that could distort ML training

---

## Contributing

We welcome contributions to any roadmap area. See [Contributing Guidelines](../contributing/README.md) for details.

To propose new features:
1. Open a GitHub issue with the `enhancement` label
2. Describe the use case and expected behavior
3. Reference relevant roadmap items if applicable

---

## Feedback

Roadmap priorities are influenced by user feedback. Please share your use cases and requirements:

- GitHub Issues: Feature requests and bug reports
- Email: [michael.ivertowski@ch.ey.com](mailto:michael.ivertowski@ch.ey.com)
