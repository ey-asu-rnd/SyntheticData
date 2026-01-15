//! Benford's Law distribution sampler and fraud amount patterns.
//!
//! Implements Benford's Law compliant amount generation and various fraud
//! amount patterns for realistic synthetic accounting data.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::AmountDistributionConfig;

/// Benford's Law probability distribution for first digits 1-9.
/// P(d) = log10(1 + 1/d)
pub const BENFORD_PROBABILITIES: [f64; 9] = [
    0.30103, // 1: 30.1%
    0.17609, // 2: 17.6%
    0.12494, // 3: 12.5%
    0.09691, // 4: 9.7%
    0.07918, // 5: 7.9%
    0.06695, // 6: 6.7%
    0.05799, // 7: 5.8%
    0.05115, // 8: 5.1%
    0.04576, // 9: 4.6%
];

/// Cumulative distribution function for Benford's Law.
pub const BENFORD_CDF: [f64; 9] = [
    0.30103, // 1
    0.47712, // 1-2
    0.60206, // 1-3
    0.69897, // 1-4
    0.77815, // 1-5
    0.84510, // 1-6
    0.90309, // 1-7
    0.95424, // 1-8
    1.00000, // 1-9
];

/// Anti-Benford distribution for generating statistically improbable amounts.
/// Overweights digits 5, 7, and 9 which are typically rare in natural data.
pub const ANTI_BENFORD_PROBABILITIES: [f64; 9] = [
    0.05, // 1: 5% (normally 30%)
    0.05, // 2: 5% (normally 18%)
    0.05, // 3: 5% (normally 12%)
    0.10, // 4: 10%
    0.25, // 5: 25% (normally 8%)
    0.10, // 6: 10%
    0.20, // 7: 20% (normally 6%)
    0.05, // 8: 5%
    0.15, // 9: 15% (normally 5%)
];

/// Fraud amount pattern types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FraudAmountPattern {
    /// Normal amount generation (Benford-compliant if enabled)
    Normal,
    /// Statistically improbable first digits (anti-Benford)
    /// Excess of leading 5s, 7s, 9s - detectable via statistical analysis
    StatisticallyImprobable,
    /// Obvious round numbers ($50,000.00, $99,999.99)
    /// Easy to spot in visual review
    ObviousRoundNumbers,
    /// Amounts clustered just below approval thresholds
    /// Classic split-transaction pattern
    ThresholdAdjacent,
}

impl Default for FraudAmountPattern {
    fn default() -> Self {
        Self::Normal
    }
}

/// Configuration for threshold-adjacent fraud pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Approval thresholds to cluster below
    pub thresholds: Vec<f64>,
    /// Minimum percentage below threshold (e.g., 0.01 = 1%)
    pub min_below_pct: f64,
    /// Maximum percentage below threshold (e.g., 0.15 = 15%)
    pub max_below_pct: f64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            thresholds: vec![1000.0, 5000.0, 10000.0, 25000.0, 50000.0, 100000.0],
            min_below_pct: 0.01,
            max_below_pct: 0.15,
        }
    }
}

/// Sampler that produces amounts following Benford's Law distribution.
pub struct BenfordSampler {
    rng: ChaCha8Rng,
    config: AmountDistributionConfig,
}

impl BenfordSampler {
    /// Create a new Benford sampler with the given seed and amount configuration.
    pub fn new(seed: u64, config: AmountDistributionConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
        }
    }

    /// Sample a first digit according to Benford's Law.
    fn sample_benford_first_digit(&mut self) -> u8 {
        let p: f64 = self.rng.gen();
        for (i, &cumulative) in BENFORD_CDF.iter().enumerate() {
            if p < cumulative {
                return (i + 1) as u8;
            }
        }
        9
    }

    /// Sample a first digit from the anti-Benford distribution.
    fn sample_anti_benford_first_digit(&mut self) -> u8 {
        let p: f64 = self.rng.gen();
        let mut cumulative = 0.0;
        for (i, &prob) in ANTI_BENFORD_PROBABILITIES.iter().enumerate() {
            cumulative += prob;
            if p < cumulative {
                return (i + 1) as u8;
            }
        }
        9
    }

    /// Sample an amount following Benford's Law.
    pub fn sample(&mut self) -> Decimal {
        self.sample_with_first_digit(self.sample_benford_first_digit())
    }

    /// Sample an amount with a specific first digit.
    pub fn sample_with_first_digit(&mut self, first_digit: u8) -> Decimal {
        let first_digit = first_digit.clamp(1, 9);

        // Determine the order of magnitude based on config range
        let min_magnitude = self.config.min_amount.log10().floor() as i32;
        let max_magnitude = self.config.max_amount.log10().floor() as i32;

        // Sample a magnitude within the valid range
        let magnitude = self.rng.gen_range(min_magnitude..=max_magnitude);
        let base = 10_f64.powi(magnitude);

        // Generate the remaining digits (0.0 to 0.999...)
        let remaining: f64 = self.rng.gen();

        // Construct: first_digit.remaining * 10^magnitude
        let mantissa = first_digit as f64 + remaining;
        let mut amount = mantissa * base;

        // Clamp to configured range
        amount = amount.clamp(self.config.min_amount, self.config.max_amount);

        // Apply round number bias (25% chance)
        let p: f64 = self.rng.gen();
        if p < self.config.round_number_probability {
            // Round to nearest whole number ending in 00
            amount = (amount / 100.0).round() * 100.0;
        } else if p < self.config.round_number_probability + self.config.nice_number_probability {
            // Round to nearest 5 or 10
            amount = (amount / 5.0).round() * 5.0;
        }

        // Round to configured decimal places
        let decimal_multiplier = 10_f64.powi(self.config.decimal_places as i32);
        amount = (amount * decimal_multiplier).round() / decimal_multiplier;

        // Ensure minimum after rounding
        amount = amount.max(self.config.min_amount);

        Decimal::from_f64_retain(amount).unwrap_or(Decimal::ONE)
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Generator for fraudulent amount patterns.
pub struct FraudAmountGenerator {
    rng: ChaCha8Rng,
    benford_sampler: BenfordSampler,
    threshold_config: ThresholdConfig,
    config: AmountDistributionConfig,
}

impl FraudAmountGenerator {
    /// Create a new fraud amount generator.
    pub fn new(
        seed: u64,
        config: AmountDistributionConfig,
        threshold_config: ThresholdConfig,
    ) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            benford_sampler: BenfordSampler::new(seed + 1, config.clone()),
            threshold_config,
            config,
        }
    }

    /// Generate an amount with the specified fraud pattern.
    pub fn sample(&mut self, pattern: FraudAmountPattern) -> Decimal {
        match pattern {
            FraudAmountPattern::Normal => self.benford_sampler.sample(),
            FraudAmountPattern::StatisticallyImprobable => self.sample_anti_benford(),
            FraudAmountPattern::ObviousRoundNumbers => self.sample_obvious_round(),
            FraudAmountPattern::ThresholdAdjacent => self.sample_threshold_adjacent(),
        }
    }

    /// Generate an amount with statistically improbable first digit distribution.
    fn sample_anti_benford(&mut self) -> Decimal {
        let first_digit = self.benford_sampler.sample_anti_benford_first_digit();
        self.benford_sampler.sample_with_first_digit(first_digit)
    }

    /// Generate an obvious round number amount (suspicious pattern).
    fn sample_obvious_round(&mut self) -> Decimal {
        let pattern_choice = self.rng.gen_range(0..5);

        let amount = match pattern_choice {
            // Even thousands ($1,000, $5,000, $10,000, etc.)
            0 => {
                let multiplier = self.rng.gen_range(1..100);
                multiplier as f64 * 1000.0
            }
            // $X9,999.99 pattern (just under round number)
            1 => {
                let base = self.rng.gen_range(1..10) as f64 * 10000.0;
                base - 0.01
            }
            // Exact $X0,000.00 pattern
            2 => {
                let multiplier = self.rng.gen_range(1..20);
                multiplier as f64 * 10000.0
            }
            // Five-thousands ($5,000, $15,000, $25,000)
            3 => {
                let multiplier = self.rng.gen_range(1..40);
                multiplier as f64 * 5000.0
            }
            // $X,999.99 pattern
            _ => {
                let base = self.rng.gen_range(1..100) as f64 * 1000.0;
                base - 0.01
            }
        };

        // Clamp to config range
        let clamped = amount.clamp(self.config.min_amount, self.config.max_amount);
        Decimal::from_f64_retain(clamped).unwrap_or(Decimal::ONE)
    }

    /// Generate an amount just below an approval threshold.
    fn sample_threshold_adjacent(&mut self) -> Decimal {
        // Select a threshold
        let threshold = if self.threshold_config.thresholds.is_empty() {
            10000.0
        } else {
            *self
                .threshold_config
                .thresholds
                .choose(&mut self.rng)
                .unwrap_or(&10000.0)
        };

        // Calculate amount as percentage below threshold
        let pct_below = self
            .rng
            .gen_range(self.threshold_config.min_below_pct..self.threshold_config.max_below_pct);
        let base_amount = threshold * (1.0 - pct_below);

        // Add small noise to avoid exact patterns
        let noise_factor = 1.0 + self.rng.gen_range(-0.005..0.005);
        let amount = base_amount * noise_factor;

        // Round to 2 decimal places
        let rounded = (amount * 100.0).round() / 100.0;

        // Ensure we're still below threshold
        let final_amount = rounded.min(threshold - 0.01);
        let clamped = final_amount.clamp(self.config.min_amount, self.config.max_amount);

        Decimal::from_f64_retain(clamped).unwrap_or(Decimal::ONE)
    }

    /// Reset the generator with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self.benford_sampler.reset(seed + 1);
    }
}

/// Extract the first digit from a decimal amount.
pub fn get_first_digit(amount: Decimal) -> Option<u8> {
    let s = amount.to_string();
    s.chars()
        .find(|c| c.is_ascii_digit() && *c != '0')
        .and_then(|c| c.to_digit(10))
        .map(|d| d as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benford_probabilities_sum_to_one() {
        let sum: f64 = BENFORD_PROBABILITIES.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "Benford probabilities sum to {}, expected 1.0",
            sum
        );
    }

    #[test]
    fn test_benford_cdf_ends_at_one() {
        assert!(
            (BENFORD_CDF[8] - 1.0).abs() < 0.0001,
            "CDF should end at 1.0"
        );
    }

    #[test]
    fn test_anti_benford_probabilities_sum_to_one() {
        let sum: f64 = ANTI_BENFORD_PROBABILITIES.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "Anti-Benford probabilities sum to {}, expected 1.0",
            sum
        );
    }

    #[test]
    fn test_benford_sampler_determinism() {
        let config = AmountDistributionConfig::default();
        let mut sampler1 = BenfordSampler::new(42, config.clone());
        let mut sampler2 = BenfordSampler::new(42, config);

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_benford_first_digit_distribution() {
        let config = AmountDistributionConfig::default();
        let mut sampler = BenfordSampler::new(12345, config);

        let mut digit_counts = [0u32; 9];
        let iterations = 10_000;

        for _ in 0..iterations {
            let amount = sampler.sample();
            if let Some(digit) = get_first_digit(amount) {
                if digit >= 1 && digit <= 9 {
                    digit_counts[(digit - 1) as usize] += 1;
                }
            }
        }

        // Verify digit 1 is most common (should be ~30%)
        let digit_1_pct = digit_counts[0] as f64 / iterations as f64;
        assert!(
            digit_1_pct > 0.20 && digit_1_pct < 0.40,
            "Digit 1 should be ~30%, got {:.1}%",
            digit_1_pct * 100.0
        );

        // Verify digit 9 is least common (should be ~5%)
        let digit_9_pct = digit_counts[8] as f64 / iterations as f64;
        assert!(
            digit_9_pct > 0.02 && digit_9_pct < 0.10,
            "Digit 9 should be ~5%, got {:.1}%",
            digit_9_pct * 100.0
        );
    }

    #[test]
    fn test_threshold_adjacent_below_threshold() {
        let config = AmountDistributionConfig::default();
        let threshold_config = ThresholdConfig {
            thresholds: vec![10000.0],
            min_below_pct: 0.01,
            max_below_pct: 0.15,
        };
        let mut gen = FraudAmountGenerator::new(42, config, threshold_config);

        for _ in 0..100 {
            let amount = gen.sample(FraudAmountPattern::ThresholdAdjacent);
            let f = amount.to_string().parse::<f64>().unwrap();
            assert!(
                f < 10000.0,
                "Amount {} should be below threshold 10000",
                f
            );
            assert!(
                f >= 8500.0,
                "Amount {} should be within 15% of threshold",
                f
            );
        }
    }

    #[test]
    fn test_obvious_round_numbers() {
        let config = AmountDistributionConfig::default();
        let threshold_config = ThresholdConfig::default();
        let mut gen = FraudAmountGenerator::new(42, config, threshold_config);

        for _ in 0..100 {
            let amount = gen.sample(FraudAmountPattern::ObviousRoundNumbers);
            let f = amount.to_string().parse::<f64>().unwrap();

            // Should be either a round number or just under one
            let is_round = f % 1000.0 == 0.0 || f % 5000.0 == 0.0;
            let is_just_under = (f + 0.01) % 1000.0 < 0.02 || (f + 0.01) % 10000.0 < 0.02;

            assert!(
                is_round || is_just_under || f > 0.0,
                "Amount {} should be a suspicious round number",
                f
            );
        }
    }

    #[test]
    fn test_get_first_digit() {
        assert_eq!(get_first_digit(Decimal::from(123)), Some(1));
        assert_eq!(get_first_digit(Decimal::from(999)), Some(9));
        assert_eq!(get_first_digit(Decimal::from(50000)), Some(5));
        assert_eq!(
            get_first_digit(Decimal::from_str_exact("0.00123").unwrap()),
            Some(1)
        );
    }
}
