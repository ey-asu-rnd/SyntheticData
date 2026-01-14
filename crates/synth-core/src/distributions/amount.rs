//! Transaction amount distribution sampler.
//!
//! Generates realistic transaction amounts using log-normal distributions
//! and round-number bias commonly observed in accounting data.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, LogNormal};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Configuration for amount distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountDistributionConfig {
    /// Minimum transaction amount
    pub min_amount: f64,
    /// Maximum transaction amount
    pub max_amount: f64,
    /// Log-normal mu parameter (location)
    pub lognormal_mu: f64,
    /// Log-normal sigma parameter (scale)
    pub lognormal_sigma: f64,
    /// Number of decimal places to round to
    pub decimal_places: u8,
    /// Probability of round number (ending in .00)
    pub round_number_probability: f64,
    /// Probability of nice number (ending in 0 or 5)
    pub nice_number_probability: f64,
}

impl Default for AmountDistributionConfig {
    fn default() -> Self {
        Self {
            min_amount: 0.01,
            max_amount: 100_000_000.0, // 100 million
            lognormal_mu: 7.0,         // Center around ~1000
            lognormal_sigma: 2.5,      // Wide spread
            decimal_places: 2,
            round_number_probability: 0.25, // 25% chance of .00 ending
            nice_number_probability: 0.15,  // 15% chance of nice numbers
        }
    }
}

impl AmountDistributionConfig {
    /// Configuration for small transactions (e.g., retail).
    pub fn small_transactions() -> Self {
        Self {
            min_amount: 0.01,
            max_amount: 10_000.0,
            lognormal_mu: 4.0, // Center around ~55
            lognormal_sigma: 1.5,
            decimal_places: 2,
            round_number_probability: 0.30,
            nice_number_probability: 0.20,
        }
    }

    /// Configuration for medium transactions (e.g., B2B).
    pub fn medium_transactions() -> Self {
        Self {
            min_amount: 100.0,
            max_amount: 1_000_000.0,
            lognormal_mu: 8.5, // Center around ~5000
            lognormal_sigma: 2.0,
            decimal_places: 2,
            round_number_probability: 0.20,
            nice_number_probability: 0.15,
        }
    }

    /// Configuration for large transactions (e.g., enterprise).
    pub fn large_transactions() -> Self {
        Self {
            min_amount: 1000.0,
            max_amount: 100_000_000.0,
            lognormal_mu: 10.0, // Center around ~22000
            lognormal_sigma: 2.5,
            decimal_places: 2,
            round_number_probability: 0.15,
            nice_number_probability: 0.10,
        }
    }
}

/// Sampler for realistic transaction amounts.
pub struct AmountSampler {
    /// RNG for sampling
    rng: ChaCha8Rng,
    /// Configuration
    config: AmountDistributionConfig,
    /// Log-normal distribution
    lognormal: LogNormal<f64>,
    /// Decimal multiplier for rounding
    decimal_multiplier: f64,
}

impl AmountSampler {
    /// Create a new sampler with default configuration.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, AmountDistributionConfig::default())
    }

    /// Create a sampler with custom configuration.
    pub fn with_config(seed: u64, config: AmountDistributionConfig) -> Self {
        let lognormal = LogNormal::new(config.lognormal_mu, config.lognormal_sigma)
            .expect("Invalid log-normal parameters");
        let decimal_multiplier = 10_f64.powi(config.decimal_places as i32);

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            lognormal,
            decimal_multiplier,
        }
    }

    /// Sample a single amount.
    pub fn sample(&mut self) -> Decimal {
        let mut amount = self.lognormal.sample(&mut self.rng);

        // Clamp to configured range
        amount = amount.clamp(self.config.min_amount, self.config.max_amount);

        // Apply round number bias
        let p: f64 = self.rng.gen();
        if p < self.config.round_number_probability {
            // Round to nearest whole number ending in 00
            amount = (amount / 100.0).round() * 100.0;
        } else if p < self.config.round_number_probability + self.config.nice_number_probability {
            // Round to nearest 5 or 10
            amount = (amount / 5.0).round() * 5.0;
        }

        // Round to configured decimal places
        amount = (amount * self.decimal_multiplier).round() / self.decimal_multiplier;

        // Ensure minimum after rounding
        amount = amount.max(self.config.min_amount);

        Decimal::from_f64_retain(amount).unwrap_or(Decimal::ONE)
    }

    /// Sample multiple amounts that sum to a target total.
    ///
    /// Useful for generating line items that must balance.
    pub fn sample_summing_to(&mut self, count: usize, total: Decimal) -> Vec<Decimal> {
        use rust_decimal::prelude::ToPrimitive;

        if count == 0 {
            return Vec::new();
        }
        if count == 1 {
            return vec![total];
        }

        let total_f64 = total.to_f64().unwrap_or(0.0);

        // Generate random weights ensuring minimum weight
        let mut weights: Vec<f64> = (0..count)
            .map(|_| self.rng.gen::<f64>().max(0.01))
            .collect();
        let sum: f64 = weights.iter().sum();
        weights.iter_mut().for_each(|w| *w /= sum);

        // Calculate amounts based on weights, using string parsing for precision
        let mut amounts: Vec<Decimal> = weights
            .iter()
            .map(|w| {
                let amount = total_f64 * w;
                let rounded = (amount * self.decimal_multiplier).round() / self.decimal_multiplier;
                // Use string format for more reliable decimal conversion
                let amount_str = format!("{:.2}", rounded);
                amount_str.parse::<Decimal>().unwrap_or(Decimal::ZERO)
            })
            .collect();

        // Adjust last amount to ensure exact sum
        let current_sum: Decimal = amounts.iter().copied().sum();
        let diff = total - current_sum;
        let last_idx = amounts.len() - 1;
        amounts[last_idx] += diff;

        // If last amount became negative (rare edge case), redistribute
        if amounts[last_idx] < Decimal::ZERO {
            let negative_amount = amounts[last_idx];
            amounts[last_idx] = Decimal::ZERO;
            // Add the negative difference to the first amount with sufficient value
            for amt in amounts.iter_mut().take(last_idx) {
                if *amt > negative_amount.abs() {
                    *amt += negative_amount;
                    break;
                }
            }
        }

        amounts
    }

    /// Sample an amount within a specific range.
    pub fn sample_in_range(&mut self, min: Decimal, max: Decimal) -> Decimal {
        let min_f64 = min.to_string().parse::<f64>().unwrap_or(0.0);
        let max_f64 = max.to_string().parse::<f64>().unwrap_or(1000000.0);

        let range = max_f64 - min_f64;
        let amount = min_f64 + self.rng.gen::<f64>() * range;

        let rounded = (amount * self.decimal_multiplier).round() / self.decimal_multiplier;
        Decimal::from_f64_retain(rounded).unwrap_or(min)
    }

    /// Reset the sampler with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng = ChaCha8Rng::seed_from_u64(seed);
    }
}

/// Sampler for currency exchange rates.
pub struct ExchangeRateSampler {
    rng: ChaCha8Rng,
    /// Base rates for common currency pairs (vs USD)
    base_rates: std::collections::HashMap<String, f64>,
    /// Daily volatility (standard deviation)
    volatility: f64,
}

impl ExchangeRateSampler {
    /// Create a new exchange rate sampler.
    pub fn new(seed: u64) -> Self {
        let mut base_rates = std::collections::HashMap::new();
        // Approximate rates as of 2024
        base_rates.insert("EUR".to_string(), 0.92);
        base_rates.insert("GBP".to_string(), 0.79);
        base_rates.insert("CHF".to_string(), 0.88);
        base_rates.insert("JPY".to_string(), 149.0);
        base_rates.insert("CNY".to_string(), 7.24);
        base_rates.insert("CAD".to_string(), 1.36);
        base_rates.insert("AUD".to_string(), 1.53);
        base_rates.insert("INR".to_string(), 83.0);
        base_rates.insert("USD".to_string(), 1.0);

        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            base_rates,
            volatility: 0.005, // 0.5% daily volatility
        }
    }

    /// Get exchange rate from one currency to another.
    pub fn get_rate(&mut self, from: &str, to: &str) -> Decimal {
        let from_usd = self.base_rates.get(from).copied().unwrap_or(1.0);
        let to_usd = self.base_rates.get(to).copied().unwrap_or(1.0);

        // Base rate
        let base_rate = to_usd / from_usd;

        // Add some random variation
        let variation = 1.0 + (self.rng.gen::<f64>() - 0.5) * 2.0 * self.volatility;
        let rate = base_rate * variation;

        // Round to 6 decimal places
        let rounded = (rate * 1_000_000.0).round() / 1_000_000.0;
        Decimal::from_f64_retain(rounded).unwrap_or(Decimal::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_sampler_determinism() {
        let mut sampler1 = AmountSampler::new(42);
        let mut sampler2 = AmountSampler::new(42);

        for _ in 0..100 {
            assert_eq!(sampler1.sample(), sampler2.sample());
        }
    }

    #[test]
    fn test_amount_sampler_range() {
        let config = AmountDistributionConfig {
            min_amount: 100.0,
            max_amount: 1000.0,
            ..Default::default()
        };
        let mut sampler = AmountSampler::with_config(42, config);

        for _ in 0..1000 {
            let amount = sampler.sample();
            let amount_f64: f64 = amount.to_string().parse().unwrap();
            assert!(amount_f64 >= 100.0, "Amount {} below minimum", amount);
            assert!(amount_f64 <= 1000.0, "Amount {} above maximum", amount);
        }
    }

    #[test]
    fn test_summing_amounts() {
        let mut sampler = AmountSampler::new(42);
        let total = Decimal::from(10000);
        let amounts = sampler.sample_summing_to(5, total);

        assert_eq!(amounts.len(), 5);

        let sum: Decimal = amounts.iter().sum();
        assert_eq!(sum, total, "Sum {} doesn't match total {}", sum, total);
    }

    #[test]
    fn test_exchange_rate() {
        let mut sampler = ExchangeRateSampler::new(42);

        let eur_usd = sampler.get_rate("EUR", "USD");
        let eur_f64: f64 = eur_usd.to_string().parse().unwrap();
        assert!(
            eur_f64 > 0.8 && eur_f64 < 1.2,
            "EUR/USD rate {} out of range",
            eur_f64
        );

        let usd_usd = sampler.get_rate("USD", "USD");
        let usd_f64: f64 = usd_usd.to_string().parse().unwrap();
        assert!(
            (usd_f64 - 1.0).abs() < 0.01,
            "USD/USD rate {} should be ~1.0",
            usd_f64
        );
    }
}
