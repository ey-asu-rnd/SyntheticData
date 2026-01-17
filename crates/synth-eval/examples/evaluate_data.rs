//! Comprehensive evaluation of synthetic data generation quality.
//!
//! Run with: cargo run -p synth-eval --example evaluate_data -- /tmp/synth-eval-output/sample_entries.json

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use synth_eval::{
    BenfordAnalyzer, AmountDistributionAnalyzer, LineItemAnalyzer, TemporalAnalyzer,
    LineItemEntry, TemporalEntry,
};

#[derive(Debug, Deserialize)]
struct JournalEntry {
    header: Header,
    lines: Vec<Line>,
}

#[derive(Debug, Deserialize)]
struct Header {
    document_id: String,
    #[allow(dead_code)]
    company_code: String,
    #[allow(dead_code)]
    fiscal_year: u16,
    #[allow(dead_code)]
    fiscal_period: u8,
    posting_date: String,
    #[allow(dead_code)]
    document_date: String,
    source: String,
    business_process: Option<String>,
    is_fraud: bool,
    #[allow(dead_code)]
    fraud_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Line {
    #[allow(dead_code)]
    line_number: u32,
    #[allow(dead_code)]
    gl_account: String,
    debit_amount: String,
    credit_amount: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let file_path = args.get(1).map(|s| s.as_str()).unwrap_or("/tmp/synth-eval-output/sample_entries.json");

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║        SYNTHETIC DATA COMPREHENSIVE EVALUATION REPORT           ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    // Load data
    println!("Loading data from: {}", file_path);
    let content = fs::read_to_string(file_path)?;
    let entries: Vec<JournalEntry> = serde_json::from_str(&content)?;

    println!("Loaded {} journal entries", entries.len());
    println!();

    // Collect all amounts for analysis
    let mut amounts: Vec<Decimal> = Vec::new();
    let mut line_item_entries: Vec<LineItemEntry> = Vec::new();
    let mut temporal_entries: Vec<TemporalEntry> = Vec::new();
    let mut balance_issues: Vec<String> = Vec::new();
    let mut fraud_count = 0;
    let mut source_counts: HashMap<String, usize> = HashMap::new();
    let mut process_counts: HashMap<String, usize> = HashMap::new();

    for entry in &entries {
        // Parse posting date
        if let Ok(date) = NaiveDate::parse_from_str(&entry.header.posting_date, "%Y-%m-%d") {
            temporal_entries.push(TemporalEntry { posting_date: date });
        }

        // Count fraud
        if entry.header.is_fraud {
            fraud_count += 1;
        }

        // Count sources
        *source_counts.entry(entry.header.source.clone()).or_insert(0) += 1;

        // Count business processes
        if let Some(ref bp) = entry.header.business_process {
            *process_counts.entry(bp.clone()).or_insert(0) += 1;
        }

        // Collect amounts and check balance
        let mut total_debit = Decimal::ZERO;
        let mut total_credit = Decimal::ZERO;
        let mut debit_count = 0usize;
        let mut credit_count = 0usize;

        for line in &entry.lines {
            if let Ok(debit) = line.debit_amount.parse::<Decimal>() {
                if debit > Decimal::ZERO {
                    amounts.push(debit);
                    total_debit += debit;
                    debit_count += 1;
                }
            }
            if let Ok(credit) = line.credit_amount.parse::<Decimal>() {
                if credit > Decimal::ZERO {
                    amounts.push(credit);
                    total_credit += credit;
                    credit_count += 1;
                }
            }
        }

        // Add line item entry
        line_item_entries.push(LineItemEntry {
            line_count: entry.lines.len(),
            debit_count,
            credit_count,
        });

        // Check if balanced
        let imbalance = (total_debit - total_credit).abs();
        if imbalance > Decimal::new(1, 2) {  // > 0.01 tolerance
            balance_issues.push(format!(
                "Entry {}: Debit={}, Credit={}, Imbalance={}",
                entry.header.document_id, total_debit, total_credit, imbalance
            ));
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // SECTION 1: BALANCE COHERENCE
    // ═══════════════════════════════════════════════════════════════════
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ 1. BALANCE COHERENCE (Debits = Credits)                        │");
    println!("└────────────────────────────────────────────────────────────────┘");

    let balance_rate = 1.0 - (balance_issues.len() as f64 / entries.len() as f64);
    let balance_status = if balance_issues.is_empty() { "✓ PASS" } else { "✗ FAIL" };

    println!("  Balance rate: {:.2}%", balance_rate * 100.0);
    println!("  Imbalanced entries: {}", balance_issues.len());
    println!("  Status: {}", balance_status);

    if !balance_issues.is_empty() && balance_issues.len() <= 5 {
        println!("  Sample issues:");
        for issue in balance_issues.iter().take(5) {
            println!("    - {}", issue);
        }
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // SECTION 2: BENFORD'S LAW ANALYSIS
    // ═══════════════════════════════════════════════════════════════════
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ 2. BENFORD'S LAW ANALYSIS                                      │");
    println!("└────────────────────────────────────────────────────────────────┘");

    let benford_analyzer = BenfordAnalyzer::new(0.05);
    let benford_result = benford_analyzer.analyze(&amounts)?;

    let benford_status = if benford_result.passes { "✓ PASS" } else { "✗ FAIL" };

    println!("  Sample size: {}", benford_result.sample_size);
    println!("  Chi-squared: {:.4}", benford_result.chi_squared);
    println!("  P-value: {:.6}", benford_result.p_value);
    println!("  MAD (Mean Absolute Deviation): {:.6}", benford_result.mad);
    println!("  Conformity level: {:?}", benford_result.conformity);
    println!("  Anti-Benford score: {:.4}", benford_result.anti_benford_score);
    println!("  Status: {}", benford_status);
    println!();

    println!("  First-Digit Distribution:");
    println!("  Digit  Expected   Observed   Deviation");
    println!("  ─────  ─────────  ─────────  ─────────");
    let expected = [0.301, 0.176, 0.125, 0.097, 0.079, 0.067, 0.058, 0.051, 0.046];
    for (i, (obs, exp)) in benford_result.observed_frequencies.iter().zip(expected.iter()).enumerate() {
        let dev = obs - exp;
        let indicator = if dev.abs() > 0.02 { "⚠" } else { "✓" };
        println!("    {}     {:.3}      {:.3}      {:+.3}  {}", i + 1, exp, obs, dev, indicator);
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // SECTION 3: AMOUNT DISTRIBUTION ANALYSIS
    // ═══════════════════════════════════════════════════════════════════
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ 3. AMOUNT DISTRIBUTION ANALYSIS                                │");
    println!("└────────────────────────────────────────────────────────────────┘");

    let amount_analyzer = AmountDistributionAnalyzer::new();
    let amount_result = amount_analyzer.analyze(&amounts)?;

    println!("  Sample size: {}", amount_result.sample_size);
    println!("  Mean: ${}", amount_result.mean);
    println!("  Median: ${}", amount_result.median);
    println!("  Std Dev: ${}", amount_result.std_dev);
    println!("  Min: ${}", amount_result.min);
    println!("  Max: ${}", amount_result.max);
    println!("  Skewness: {:.4}", amount_result.skewness);
    println!("  Kurtosis: {:.4}", amount_result.kurtosis);
    println!();
    println!("  Round number ratio: {:.2}%", amount_result.round_number_ratio * 100.0);
    println!("  Nice number ratio: {:.2}%", amount_result.nice_number_ratio * 100.0);
    if let Some(p) = amount_result.lognormal_ks_pvalue {
        let status = if p > 0.05 { "✓ PASS" } else { "⚠ MARGINAL" };
        println!("  Log-normal KS p-value: {:.6} {}", p, status);
    }
    if let (Some(mu), Some(sigma)) = (amount_result.fitted_mu, amount_result.fitted_sigma) {
        println!("  Fitted log-normal: μ={:.4}, σ={:.4}", mu, sigma);
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // SECTION 4: LINE ITEM DISTRIBUTION ANALYSIS
    // ═══════════════════════════════════════════════════════════════════
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ 4. LINE ITEM DISTRIBUTION ANALYSIS                             │");
    println!("└────────────────────────────────────────────────────────────────┘");

    let line_analyzer = LineItemAnalyzer::new(0.05);
    let line_result = line_analyzer.analyze(&line_item_entries)?;

    println!("  Sample size: {}", line_result.sample_size);
    println!("  Average line items: {:.2}", line_result.avg_line_count);
    println!("  Min line count: {}", line_result.min_line_count);
    println!("  Max line count: {}", line_result.max_line_count);
    println!("  Even ratio: {:.2}% (expected: 88%)", line_result.even_ratio * 100.0);
    println!("  Even ratio deviation: {:.4}", line_result.even_ratio_deviation);
    println!("  Equal split ratio: {:.2}% (expected: 82%)", line_result.equal_split_ratio * 100.0);
    println!("  Chi-squared: {:.4}", line_result.chi_squared);
    println!("  P-value: {:.6}", line_result.p_value);
    let line_status = if line_result.passes { "✓ PASS" } else { "✗ FAIL" };
    println!("  Status: {}", line_status);
    println!();

    // Show distribution
    println!("  Line Count Distribution:");
    println!("  Count    Observed    Expected");
    println!("  ─────    ────────    ────────");
    let expected_dist = [(2, 0.6068), (3, 0.0577), (4, 0.1663), (5, 0.0306), (6, 0.0332)];
    let total = line_result.sample_size as f64;
    for (count, expected_pct) in &expected_dist {
        let observed = line_result.line_count_distribution.get(count).unwrap_or(&0);
        let observed_pct = *observed as f64 / total;
        let indicator = if (observed_pct - expected_pct).abs() > 0.05 { "⚠" } else { "✓" };
        println!("  {:5}    {:.4}      {:.4}    {}", count, observed_pct, expected_pct, indicator);
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // SECTION 5: TEMPORAL PATTERN ANALYSIS
    // ═══════════════════════════════════════════════════════════════════
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ 5. TEMPORAL PATTERN ANALYSIS                                   │");
    println!("└────────────────────────────────────────────────────────────────┘");

    let temporal_analyzer = TemporalAnalyzer::new();
    let temporal_result = temporal_analyzer.analyze(&temporal_entries)?;

    println!("  Sample size: {}", temporal_result.sample_size);
    println!("  Date range: {} to {}", temporal_result.start_date, temporal_result.end_date);
    println!("  Days spanned: {}", temporal_result.days_spanned);
    println!("  Weekend activity ratio: {:.2}% (expected: <10%)", temporal_result.weekend_ratio * 100.0);
    println!("  Month-end spike: {:.2}x (expected: ~2.5x)", temporal_result.month_end_spike);
    println!("  Quarter-end spike: {:.2}x (expected: ~4.0x)", temporal_result.quarter_end_spike);
    println!("  Year-end spike: {:.2}x (expected: ~6.0x)", temporal_result.year_end_spike);
    println!("  Pattern correlation: {:.4}", temporal_result.pattern_correlation);
    println!("  Day-of-week correlation: {:.4}", temporal_result.day_of_week_correlation);
    let temporal_status = if temporal_result.passes { "✓ PASS" } else { "✗ FAIL" };
    println!("  Status: {}", temporal_status);
    println!();

    println!("  Day of Week Distribution:");
    println!("  Day          Volume");
    println!("  ──────────   ──────");
    let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
    for day in &days {
        let vol = temporal_result.day_of_week_distribution.get(*day).unwrap_or(&0.0);
        println!("  {:10}   {:.4}", day, vol);
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // SECTION 6: DATA COMPOSITION
    // ═══════════════════════════════════════════════════════════════════
    println!("┌────────────────────────────────────────────────────────────────┐");
    println!("│ 6. DATA COMPOSITION                                            │");
    println!("└────────────────────────────────────────────────────────────────┘");

    let total = entries.len();
    println!("  Fraud entries: {} ({:.2}%)", fraud_count,
             (fraud_count as f64 / total as f64) * 100.0);
    println!();

    println!("  Source Distribution:");
    for (source, count) in &source_counts {
        println!("    {:15}: {:6} ({:.1}%)", source, count, (*count as f64 / total as f64) * 100.0);
    }
    println!();

    println!("  Business Process Distribution:");
    for (process, count) in &process_counts {
        println!("    {:15}: {:6} ({:.1}%)", process, count, (*count as f64 / total as f64) * 100.0);
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // SECTION 7: OVERALL EVALUATION SUMMARY
    // ═══════════════════════════════════════════════════════════════════
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                    EVALUATION SUMMARY                            ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    let mut issues: Vec<(String, &str)> = Vec::new();
    let mut score: f64 = 100.0;

    // Balance check
    if !balance_issues.is_empty() {
        issues.push(("Balance coherence: Some entries are not balanced".to_string(), "CRITICAL"));
        score -= 20.0;
    }

    // Benford check
    if !benford_result.passes {
        issues.push((format!("Benford's Law: p-value {:.4} < 0.05", benford_result.p_value), "HIGH"));
        score -= 15.0;
    }

    // MAD check
    if benford_result.mad > 0.015 {
        issues.push((format!("Benford MAD: {:.4} > 0.015 (acceptable)", benford_result.mad), "MEDIUM"));
        score -= 10.0;
    }

    // Line item check
    if !line_result.passes {
        issues.push(("Line item distribution doesn't match expected pattern".to_string(), "MEDIUM"));
        score -= 10.0;
    }

    // Even ratio check
    let even_diff = (line_result.even_ratio - 0.88).abs();
    if even_diff > 0.10 {
        issues.push((format!("Even/odd ratio: {:.1}% vs expected 88%", line_result.even_ratio * 100.0), "MEDIUM"));
        score -= 5.0;
    }

    // Weekend ratio check
    if temporal_result.weekend_ratio > 0.15 {
        issues.push((format!("Weekend activity: {:.1}% (expected < 10%)", temporal_result.weekend_ratio * 100.0), "MEDIUM"));
        score -= 5.0;
    }

    // Month-end spike check
    if temporal_result.month_end_spike < 1.5 {
        issues.push((format!("Month-end spike: {:.1}x (expected ~2.5x)", temporal_result.month_end_spike), "LOW"));
        score -= 5.0;
    }

    // Temporal pattern check
    if !temporal_result.passes {
        issues.push(("Temporal patterns don't match expectations".to_string(), "MEDIUM"));
        score -= 5.0;
    }

    if issues.is_empty() {
        println!("  ✓ All checks passed!");
        println!("  Score: {:.0}/100", score);
    } else {
        println!("  Issues found: {}", issues.len());
        println!();
        for (issue, severity) in &issues {
            let icon = match *severity {
                "CRITICAL" => "🔴",
                "HIGH" => "🟠",
                "MEDIUM" => "🟡",
                _ => "🟢",
            };
            println!("  {} [{}] {}", icon, severity, issue);
        }
        println!();
        println!("  Overall Score: {:.0}/100", score.max(0.0));
    }
    println!();

    // ═══════════════════════════════════════════════════════════════════
    // SECTION 8: ENHANCEMENT RECOMMENDATIONS
    // ═══════════════════════════════════════════════════════════════════
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║                  ENHANCEMENT RECOMMENDATIONS                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    let mut recommendations: Vec<(&str, &str, &str)> = Vec::new();

    if !balance_issues.is_empty() {
        recommendations.push((
            "CRITICAL",
            "Balance Validation",
            "Fix balance coherence - all journal entries MUST have debits = credits"
        ));
    }

    if !benford_result.passes || benford_result.mad > 0.012 {
        recommendations.push((
            "HIGH",
            "Benford's Law Compliance",
            "Tune amount distribution parameters to better match Benford's Law.\n\
             Consider adjusting lognormal_mu and lognormal_sigma parameters."
        ));
    }

    if !line_result.passes {
        recommendations.push((
            "MEDIUM",
            "Line Item Distribution",
            "Adjust line item distribution parameters to better match empirical data.\n\
             Expected: 60% 2-line, 17% 4-line entries. Review line_item_distribution config."
        ));
    }

    if (line_result.even_ratio - 0.88).abs() > 0.05 {
        recommendations.push((
            "MEDIUM",
            "Even/Odd Line Ratio",
            "Adjust even/odd line item ratio. Research suggests 88% even, 12% odd.\n\
             Review even_odd_distribution settings."
        ));
    }

    if temporal_result.weekend_ratio > 0.10 {
        recommendations.push((
            "MEDIUM",
            "Weekend Activity",
            "Reduce weekend transaction volume. Most business transactions occur on weekdays.\n\
             Adjust weekend_activity parameter to ~5%."
        ));
    }

    if temporal_result.month_end_spike < 2.0 {
        recommendations.push((
            "LOW",
            "Month-End Patterns",
            "Strengthen month-end spike patterns for more realistic temporal distribution.\n\
             Set month_end_multiplier to 2.5-3.0."
        ));
    }

    if benford_result.anti_benford_score > 0.3 {
        recommendations.push((
            "HIGH",
            "Anti-Benford Patterns",
            "High anti-Benford score indicates potential fraud-like patterns.\n\
             If unintentional, review amount generation for anomalies."
        ));
    }

    if amount_result.skewness > 5.0 {
        recommendations.push((
            "LOW",
            "Amount Distribution Skewness",
            "Very high skewness in amounts. Consider adjusting max_amount\n\
             or using a different distribution shape."
        ));
    }

    if recommendations.is_empty() {
        println!("  ✓ No major enhancements needed - generation quality is excellent!");
    } else {
        for (priority, area, recommendation) in &recommendations {
            let icon = match *priority {
                "CRITICAL" => "🔴",
                "HIGH" => "🟠",
                "MEDIUM" => "🟡",
                _ => "🟢",
            };
            println!("  {} [{}] {}", icon, priority, area);
            for line in recommendation.lines() {
                println!("     {}", line);
            }
            println!();
        }
    }

    println!("══════════════════════════════════════════════════════════════════════");
    println!("                        END OF EVALUATION REPORT");
    println!("══════════════════════════════════════════════════════════════════════");

    Ok(())
}
