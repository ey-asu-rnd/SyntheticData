# Comprehensive Specification — Synthetic Financial Transaction Data Generator
*(Realistic behavior + coherent ecosystems + ML ground truth, aligned with KYC/AML regulatory expectations)*

## 1) Purpose & Intended Outcomes

This specification defines a **conceptual** (non-technical) design for a synthetic data generator that produces **high-fidelity financial transaction data** and **multi-level ground truth** suitable for:

- **KYC/CDD** training and testing (customer profiling, expected activity, refresh triggers)
- **AML/CFT transaction monitoring** development (typology detection, alert tuning, SAR casework simulation)
- **Customer & entity risk classification** (RBA segmentation, scorecards, portfolio risk)
- **Fraud analytics** (retail and corporate patterns)
- **Spoofing / adversarial robustness** evaluation (evasive behavior that appears legitimate)
- **Model governance** exercises (bias, explainability, drift testing)

The generator’s core promise is **realism through causality + regulatory coherence**, not just random plausible-looking transactions.

---

## 2) Regulatory Alignment (Conceptual Anchors)

The generator must support a **risk-based approach** and **CDD/ongoing monitoring** expectations consistent with FATF-style standards, including maintaining customer information up to date and scrutinizing transactions against the customer profile.  [oai_citation:0‡fatf-gafi.org](https://www.fatf-gafi.org/content/dam/fatf-gafi/recommendations/FATF%20Recommendations%202012.pdf.coredownload.inline.pdf?utm_source=chatgpt.com)

It must also support **beneficial ownership transparency** requirements for legal entities and legal arrangements (including trusts), including ownership/control modeling and data needed for onboarding and monitoring.  [oai_citation:1‡EUR-Lex](https://eur-lex.europa.eu/EN/legal-content/summary/preventing-abuse-of-the-financial-system-for-money-laundering-and-terrorism-purposes-until-2027.html?utm_source=chatgpt.com)

**Swiss realism mode** (optional configuration) must align conceptually with the Swiss AML framework and FINMA supervision expectations, including due diligence and prevention of misuse of the financial system.  [oai_citation:2‡Eidgenössische Finanzmarktaufsicht FINMA](https://www.finma.ch/en/supervision/cross-sector-issues/combating-money-laundering/?utm_source=chatgpt.com)  
Additionally, the generator should support outputs consistent with the Swiss Banking Association’s due diligence practice framework (CDB 20).  [oai_citation:3‡Swiss Bankers Association](https://www.swissbanking.ch/_Resources/Persistent/6/2/e/e/62eec3df0685e359c5a376dfca79dec8b908ea9c/SBA_Agreement_CDB_2020_EN.pdf?utm_source=chatgpt.com)

> **Design principle:** every “suspicious” or “high-risk” output must be explainable as a *violation of expected activity / economic rationale* within a risk-based compliance framework.  [oai_citation:4‡fatf-gafi.org](https://www.fatf-gafi.org/content/dam/fatf-gafi/recommendations/FATF%20Recommendations%202012.pdf.coredownload.inline.pdf?utm_source=chatgpt.com)

---

## 3) Scope & Non-Goals

### In scope
- Retail + business + trust-like customer ecosystems
- Accounts, counterparties, merchants, and relationship graphs
- Temporal realism (calendars, posting/settlement, seasonality, life events)
- Statistical realism (Benford where appropriate, heavy tails, price clustering)
- Fraud + AML typologies and their ground truth labels
- KYC profiles, expected behavior, and risk classification ground truth

### Not in scope
- Implementation/architecture, data pipelines, storage formats, performance engineering
- Direct “how-to evade detection” guidance (the generator is for defense/testing)

---

## 4) Core Realism Requirements

The generator must be realistic at **five layers simultaneously**:

1. **Transaction realism**: believable amounts, timestamps, channels, references  
2. **Account realism**: balance feasibility, recurring flows, product constraints  
3. **Customer realism**: budget logic, stated purpose, life/business drivers  
4. **Network realism**: shared merchants, community clusters, supply chains  
5. **Portfolio realism**: segment distributions, seasonality, macro effects  

**Causality requirement:** each recurring or material pattern must have a driver (salary, rent, invoice cycle, taxation, trust distributions, etc.).  
**Compliance requirement:** the customer’s KYC profile must define the “expected activity envelope” used for anomaly labeling and monitoring evaluation.  [oai_citation:5‡fatf-gafi.org](https://www.fatf-gafi.org/content/dam/fatf-gafi/recommendations/FATF%20Recommendations%202012.pdf.coredownload.inline.pdf?utm_source=chatgpt.com)

---

## 5) Entities & Personas (Behavioral Blueprints)

Each persona is defined by:
- **Identity model** (person/entity type, residence/industry, relationships)
- **Economic model** (income sources, cost structure, liquidity constraints)
- **Channel model** (cash intensity, digital usage, cross-border propensity)
- **Rhythm model** (temporal cadence: weekly/monthly/seasonal)
- **Risk model** (baseline risk tier, optional elevated-risk conditions)

### 5.1 Natural persons (retail)
Representative personas:
- Student / early career / mid-career / retiree
- Cross-border commuter
- Gig worker (irregular income)
- HNW individual (diversified inflows/outflows)
- Cash-heavy lifestyle vs mostly cashless

Behavioral features:
- Salary and bill cycles
- Price-point clustering for card spend
- Household transfers and shared expenses
- Occasional shocks (medical, travel, relocation)

### 5.2 Families / households
- Multiple members, joint + personal accounts
- Structured household budget and monthly rhythm
- Childcare, insurance, rent/mortgage, education
- Family support transfers, inheritance events

### 5.3 Companies (legal entities)
Personas across size tiers:

**Micro / SME**
- Narrow supplier base, strong seasonality
- Payroll timing, recurring taxes/fees
- Merchant acquiring inflows (if retail business)

**Mid-market**
- Broader AP/AR, approvals behavior (conceptual)
- Multi-bank patterns and treasury flows
- Expense reimbursements and corporate cards

**Large enterprise**
- High-volume AP/AR, intercompany flows
- Shared services, vendor master dynamics
- Highly regular payroll and tax patterns

### 5.4 Trusts / foundations / legal arrangements
- Settlor, trustee, beneficiaries, protectors (optional)
- Distributions, fees, investment-related cash legs
- Higher opacity, cross-border likelihood
- Beneficial ownership/control truth must be explicit in ground truth outputs.  [oai_citation:6‡EUR-Lex](https://eur-lex.europa.eu/EN/legal-content/summary/preventing-abuse-of-the-financial-system-for-money-laundering-and-terrorism-purposes-until-2027.html?utm_source=chatgpt.com)

### 5.5 Counterparties & ecosystem actors
- Employers (payroll hubs)
- Utilities/telecom hubs
- Major e-commerce merchants
- Government/tax entities
- Professional services (law, accounting)
- Financial institutions & payment intermediaries (conceptual)

---

## 6) Products, Accounts & Transaction Types

### 6.1 Account types
- Retail: current, savings
- Business: operating, payroll, escrow-like
- Lending: loan repayment accounts (principal + interest)
- Investment cash legs (optional)
- Wallet/prepaid (optional)

### 6.2 Transaction rails (conceptual)
- Account-to-account transfers (domestic/cross-border)
- Card payments with merchant categories
- Direct debit / standing orders
- Cash deposits/withdrawals (ATM/branch)
- Refunds, reversals, chargebacks (recommended for realism)

### 6.3 Mandatory transaction fields (semantic)
- Unique identifiers (transaction, account, customer/entity)
- Timestamps: initiated, booked, settled
- Amount, currency, FX info (if applicable)
- Direction (debit/credit)
- Channel (cash/card/transfer/etc.)
- Counterparty/merchant identifiers
- Free-text references (invoice-like patterns)
- Purpose/category codes (internal taxonomy + MCC-like)
- Optional: balance-before/after or feasibility flags

---

## 7) Temporal Constraints & Calendar Logic

The generator must implement **time realism**, including:

### 7.1 Posting/settlement dynamics
- Initiation vs booking vs settlement delays
- Weekend/holiday impacts (configurable)
- Reversals and delayed corrections

### 7.2 Cadence patterns
Retail:
- Salary at end/beginning of month
- Rent/mortgage, insurance, subscriptions
- Grocery/transport weekly cycles

Corporate:
- Payroll schedule (weekly/biweekly/monthly)
- Supplier payment terms (e.g., net 30)
- Tax/VAT remittance cycles

### 7.3 Seasonality
- Holiday spending spikes
- Travel seasons
- Back-to-school expenses
- Year-end corporate effects (bonuses, closing activities)

### 7.4 Life/business events (controlled discontinuities)
Examples:
- job change (new payroll hub)
- birth/marriage/divorce
- loan origination + repayment stream
- business expansion (new vendors, higher payroll)
- trust beneficiary changes

**Ground truth must include the causal event** so models can learn “legitimate change” vs “suspicious change”.

---

## 8) Statistical Distributions & Amount Realism

The generator must produce **mixtures**, not single distributions, and must support **segment-dependent parameterization**.

### 8.1 Amount distributions (multi-modal mixtures)
- Retail discretionary spend: lognormal-like with truncation
- Bills: fixed or narrow band
- Salary: stable with occasional bonuses
- Corporate invoices: heavy-tailed (few large, many medium/small)
- Cash withdrawals: discrete steps (20/50/100 multiples)

### 8.2 Benford’s Law (selective, context-aware)
Support configurable Benford adherence for:
- invoice-like payments and accounting-originated amounts  
Avoid forcing Benford on:
- retail card payments (often price-point clustered)
- cash withdrawals (denomination-driven)

Controls:
- `benford_strength` by transaction family
- `rounding_probability` by persona/merchant type
- `price_point_clustering_strength` for retail spend

### 8.3 Correlated realism
Amounts and categories must correlate with:
- income/turnover band
- household size
- geography/cost of living
- industry and company size
- product/channel usage and risk tier

### 8.4 Currency & FX realism (optional mode)
- corridor choices influenced by persona/industry
- fee/markup patterns
- repeated base-currency budgeting behavior

---

## 9) Coherence & Interconnectivity (Graph Realism)

### 9.1 Relationship graph
The dataset must embed a **transaction network** with:
- recurring counterparties (utilities, telecom, landlord, suppliers)
- shared merchants across many customers (realistic hubs)
- community clusters (same employer, neighborhood patterns)
- household transfer networks
- supply-chain structures (B2B vendor/customer graphs)

Expected graph properties:
- hub concentration (large merchants/employers)
- clustered communities (family/workplace)
- bridges (professional services, payroll providers)

### 9.2 Flow integrity
Even without full accounting double-entry, flows must obey:
- balance feasibility rules (or explicit overdraft logic)
- typical “float” and settlement effects
- refund/chargeback logic (optional but valuable)

---

## 10) KYC Profiles, Expected Activity & Risk Segmentation

Each customer/entity must have an explicit **KYC profile ground truth**:
- declared purpose/nature of relationship
- expected activity (volume, frequency, categories)
- source of funds / source of wealth categories
- geographic exposure and corridors
- cash intensity expectation
- occupation/industry (for companies: business model)
- beneficial owners, controllers, signatories (entities/trusts)  [oai_citation:7‡EUR-Lex](https://eur-lex.europa.eu/EN/legal-content/summary/preventing-abuse-of-the-financial-system-for-money-laundering-and-terrorism-purposes-until-2027.html?utm_source=chatgpt.com)

This enables **expected behavior envelopes** required by risk-based ongoing monitoring.  [oai_citation:8‡fatf-gafi.org](https://www.fatf-gafi.org/content/dam/fatf-gafi/recommendations/FATF%20Recommendations%202012.pdf.coredownload.inline.pdf?utm_source=chatgpt.com)

### 10.1 Risk classification outputs (ground truth)
- customer/entity risk tier (low/medium/high)
- contributing risk factors (jurisdiction/product/channel/complexity)
- volatility expectations (stable vs irregular income/turnover)
- beneficial ownership complexity score

---

## 11) Fraud & AML Typology Injection (Scenario-Based)

### 11.1 Design philosophy
Fraud and laundering patterns must be:
- **narrative-driven** (motive and economic rationale exist)
- **stage-aware** (placement → layering → integration)
- **detectability-controllable** (obvious ↔ subtle ↔ adversarial)
- **cluster-consistent** (not isolated single transactions without context)

### 11.2 Fraud patterns (examples)
Retail:
- account takeover (spend spike + new merchant set)
- first-party abuse (refund/chargeback anomalies)
- mule onboarding pattern (many micro-inbounds, rapid cash-out)

Corporate:
- fake vendor + invoice payments
- invoice splitting below approval thresholds
- ghost employees (payroll anomalies)
- business email compromise-like urgent payments (conceptual)

### 11.3 AML laundering patterns (examples)
- structuring/smurfing (sub-threshold deposits then consolidation)
- funnel accounts (many unrelated inbounds → rapid outward transfers)
- layering chains (slicing + time jitter + multiple hops)
- round-tripping (funds leave and return via affiliates)
- trust/entity opacity patterns (complex control chains)  [oai_citation:9‡EUR-Lex](https://eur-lex.europa.eu/EN/legal-content/summary/preventing-abuse-of-the-financial-system-for-money-laundering-and-terrorism-purposes-until-2027.html?utm_source=chatgpt.com)

### 11.4 Scenario attributes to generate as ground truth
For each scenario:
- typology name and variant
- laundering stage(s)
- start/end window
- involved entities/accounts/relationships
- evasion tactic class (thresholding, time jitter, proxying, cover traffic)
- “case grouping key” (transactions belonging to same case)

---

## 12) Spoofing / Adversarial Robustness Mode

To stress-test monitoring systems, the generator must support **“looks normal” laundering/fraud**:

- transactions aligned to normal salary/billing cycles
- amounts sampled close to the customer’s normal distribution
- merchant categories consistent with persona
- “cover traffic” inserted between illicit hops
- longer dwell times and reduced velocity signatures

Ground truth must include:
- spoofing intensity level (0..1)
- which dimensions were spoofed (timing/amount/merchant/counterparty)
- expected failure mode of naive rules (for evaluation)

> This is for **defensive testing and model hardening**, aligned to the expectation that controls evolve under a risk-based approach.  [oai_citation:10‡fatf-gafi.org](https://www.fatf-gafi.org/content/dam/fatf-gafi/recommendations/FATF%20Recommendations%202012.pdf.coredownload.inline.pdf?utm_source=chatgpt.com)

---

## 13) Ground Truth Outputs (Multi-Level Labels)

The generator must emit ground truth for **transaction**, **entity**, and **network** learning tasks.

### 13.1 Transaction-level ground truth
- `is_suspicious` (binary)
- `suspicion_reason` (typology class)
- `laundering_stage` (placement/layering/integration/other)
- `case_id` (links to scenario narrative)
- `expected_alert_family` (rule-based archetype mapping)
- `is_spoofed` + `spoofing_intensity`

### 13.2 Entity-level ground truth
- `customer_risk_tier`
- `expected_monthly_turnover_band`
- `cash_intensity_expected`
- `true_source_of_funds` vs `declared_source_of_funds` (when deception modeled)
- `beneficial_owner_complexity_score`  [oai_citation:11‡EUR-Lex](https://eur-lex.europa.eu/EN/legal-content/summary/preventing-abuse-of-the-financial-system-for-money-laundering-and-terrorism-purposes-until-2027.html?utm_source=chatgpt.com)

### 13.3 Relationship-level ground truth
- relationship type: family / employer / vendor / trustee-beneficiary / etc.
- `is_related_party`
- `is_mule_link` / `is_shell_link` (when applicable)
- ownership/control edges for beneficial ownership truth

### 13.4 Case narrative ground truth (human-auditable)
For each suspicious case:
- short storyline (1–5 sentences)
- key evidence points (features the investigator would notice)
- violated expectation (profile mismatch dimension)
- recommended conceptual outcome: review / EDD / escalate

---

## 14) Quality Constraints & Validation Criteria

### 14.1 Hard constraints
- no impossible balances unless overdraft is enabled
- timestamps in valid order (initiation ≤ booking ≤ settlement)
- coherent customer/entity identity & relationships
- beneficial owner/control structures are consistent and resolvable  [oai_citation:12‡EUR-Lex](https://eur-lex.europa.eu/EN/legal-content/summary/preventing-abuse-of-the-financial-system-for-money-laundering-and-terrorism-purposes-until-2027.html?utm_source=chatgpt.com)

### 14.2 Soft constraints (portfolio plausibility)
- stable merchant sets with gradual drift
- predictable bill cadence with occasional disruptions
- employer/payroll hubs create realistic community structure
- corporate vendor concentration varies by industry and size

### 14.3 Portfolio realism report (required artifact)
- distribution snapshots by segment (amounts, categories, channels)
- seasonality indicators by persona
- network metrics (hub concentration, clustering)
- baseline alert volume simulation guidance (conceptual)

---

## 15) Configuration Surface (Scenario-Intent Driven)

Configuration must be expressed in **business terms**, not technical knobs:

### 15.1 Population
- number of customers per persona
- household formation rate
- company industry mix and size distribution
- trust/foundation prevalence

### 15.2 Product/channel
- cash usage intensity by segment
- cross-border enablement level
- card vs transfer dominance

### 15.3 Risk & compliance posture
- risk appetite (share of high-risk relationships)
- KYC completeness variability
- refresh intensity for higher-risk segments (conceptual)  [oai_citation:13‡fatf-gafi.org](https://www.fatf-gafi.org/content/dam/fatf-gafi/recommendations/FATF%20Recommendations%202012.pdf.coredownload.inline.pdf?utm_source=chatgpt.com)

### 15.4 Crime injection controls
- prevalence per typology
- sophistication tiers (basic ↔ professional)
- detectability setting (obvious ↔ stealthy)
- scenario duration (one-off ↔ long-running)

---

## 16) Required Output Artifacts (Conceptual Deliverables)

The generator must produce:

1. **Transactions dataset**
2. **Customer/entity KYC dataset** (profiles + expected activity envelope)
3. **Relationship & ownership/control graph dataset**
4. **Ground truth labels** (transaction/entity/relationship/case)
5. **Case narratives dataset** (investigator-style summaries)
6. **Validation & realism report**

---

## 17) Minimum Scenario Library (Coverage Matrix)

### 17.1 Legitimate scenarios
- stable salary + bills + discretionary spend
- student with parental support cadence
- family with childcare + mortgage + insurance
- SME with merchant acquiring + suppliers + payroll
- consulting firm with irregular client payments + taxes
- trust with periodic distributions and fees

### 17.2 Fraud/AML scenarios
- structuring + consolidation
- funnel account cash-out
- layering chain with cover traffic
- round-tripping via related entities
- fake vendor invoicing + payment splitting
- mule network recruitment dynamics
- spoofed “low-and-slow” laundering behavior

---

## 18) Success Criteria (“Very Realistic” Definition)

The generator is successful when:

- compliance/ops experts judge samples as **plausible** and **internally coherent**
- networks show **realistic interconnectivity** (hubs + communities)
- risk tiers align with **CDD expectations and ownership complexity**  [oai_citation:14‡EUR-Lex](https://eur-lex.europa.eu/EN/legal-content/summary/preventing-abuse-of-the-financial-system-for-money-laundering-and-terrorism-purposes-until-2027.html?utm_source=chatgpt.com)
- suspicious cases are explainable as **profile/behavior mismatches**
- ML models trained on labels learn meaningful features beyond trivial cues
- spoofing mode reliably exposes weaknesses in naive rule sets

---

## 19) Ethical & Safe Use Boundaries

This generator is intended for:
- defensive monitoring development
- control validation and robustness testing
- training and benchmarking with explainable ground truth

It must **not** be used as a guide for wrongdoing. “Spoofing mode” exists solely to evaluate and improve defensive detection under a risk-based compliance model.  [oai_citation:15‡fatf-gafi.org](https://www.fatf-gafi.org/content/dam/fatf-gafi/recommendations/FATF%20Recommendations%202012.pdf.coredownload.inline.pdf?utm_source=chatgpt.com)

---

## Appendix — Regulatory Reference Pointers (Non-Exhaustive)

- FATF Recommendations / standards: CDD, ongoing monitoring, RBA  [oai_citation:16‡fatf-gafi.org](https://www.fatf-gafi.org/content/dam/fatf-gafi/recommendations/FATF%20Recommendations%202012.pdf.coredownload.inline.pdf?utm_source=chatgpt.com)  
- EU AML framework summary: beneficial ownership transparency and central registers  [oai_citation:17‡EUR-Lex](https://eur-lex.europa.eu/EN/legal-content/summary/preventing-abuse-of-the-financial-system-for-money-laundering-and-terrorism-purposes-until-2027.html?utm_source=chatgpt.com)  
- FINMA overview and legal basis references (Swiss AML supervision context)  [oai_citation:18‡Eidgenössische Finanzmarktaufsicht FINMA](https://www.finma.ch/en/supervision/cross-sector-issues/combating-money-laundering/?utm_source=chatgpt.com)  
- Swiss Bankers Association CDB 20 due diligence agreement and commentary  [oai_citation:19‡Swiss Bankers Association](https://www.swissbanking.ch/_Resources/Persistent/6/2/e/e/62eec3df0685e359c5a376dfca79dec8b908ea9c/SBA_Agreement_CDB_2020_EN.pdf?utm_source=chatgpt.com)  