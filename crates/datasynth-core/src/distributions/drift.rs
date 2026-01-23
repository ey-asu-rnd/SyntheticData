//! Temporal drift simulation for realistic data distribution evolution.
//!
//! Implements gradual, sudden, and seasonal drift patterns commonly observed
//! in real-world enterprise data, useful for training drift detection models.

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Types of temporal drift patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DriftType {
    /// Gradual, continuous drift over time (like inflation).
    #[default]
    Gradual,
    /// Sudden, point-in-time shifts (like policy changes).
    Sudden,
    /// Recurring patterns that cycle (like seasonal variations).
    Recurring,
    /// Combination of gradual background drift with occasional sudden shifts.
    Mixed,
}

/// Configuration for temporal drift simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfig {
    /// Enable temporal drift simulation.
    pub enabled: bool,
    /// Amount mean drift per period (e.g., 0.02 = 2% shift per month).
    pub amount_mean_drift: f64,
    /// Amount variance drift per period.
    pub amount_variance_drift: f64,
    /// Anomaly rate drift per period.
    pub anomaly_rate_drift: f64,
    /// Concept drift rate (0.0-1.0).
    pub concept_drift_rate: f64,
    /// Probability of sudden drift in any period.
    pub sudden_drift_probability: f64,
    /// Magnitude of sudden drift events.
    pub sudden_drift_magnitude: f64,
    /// Enable seasonal drift patterns.
    pub seasonal_drift: bool,
    /// Period to start drift (0 = from beginning).
    pub drift_start_period: u32,
    /// Type of drift pattern.
    pub drift_type: DriftType,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            amount_mean_drift: 0.02,
            amount_variance_drift: 0.0,
            anomaly_rate_drift: 0.0,
            concept_drift_rate: 0.01,
            sudden_drift_probability: 0.0,
            sudden_drift_magnitude: 2.0,
            seasonal_drift: false,
            drift_start_period: 0,
            drift_type: DriftType::Gradual,
        }
    }
}

/// Drift adjustments computed for a specific period.
#[derive(Debug, Clone, Default)]
pub struct DriftAdjustments {
    /// Multiplier for amount mean (1.0 = no change).
    pub amount_mean_multiplier: f64,
    /// Multiplier for amount variance (1.0 = no change).
    pub amount_variance_multiplier: f64,
    /// Additive adjustment to anomaly rate.
    pub anomaly_rate_adjustment: f64,
    /// Overall concept drift factor (0.0-1.0).
    pub concept_drift_factor: f64,
    /// Whether a sudden drift event occurred.
    pub sudden_drift_occurred: bool,
    /// Seasonal factor (1.0 = baseline, varies by month).
    pub seasonal_factor: f64,
}

impl DriftAdjustments {
    /// No drift (identity adjustments).
    pub fn none() -> Self {
        Self {
            amount_mean_multiplier: 1.0,
            amount_variance_multiplier: 1.0,
            anomaly_rate_adjustment: 0.0,
            concept_drift_factor: 0.0,
            sudden_drift_occurred: false,
            seasonal_factor: 1.0,
        }
    }
}

/// Controller for computing and applying temporal drift.
pub struct DriftController {
    config: DriftConfig,
    rng: ChaCha8Rng,
    /// Track which periods had sudden drift events for reproducibility.
    sudden_drift_periods: Vec<u32>,
    /// Total periods in the simulation.
    total_periods: u32,
}

impl DriftController {
    /// Create a new drift controller with the given configuration.
    pub fn new(config: DriftConfig, seed: u64, total_periods: u32) -> Self {
        let mut controller = Self {
            config,
            rng: ChaCha8Rng::seed_from_u64(seed),
            sudden_drift_periods: Vec::new(),
            total_periods,
        };

        // Pre-compute sudden drift events for reproducibility
        if controller.config.enabled
            && (controller.config.drift_type == DriftType::Sudden
                || controller.config.drift_type == DriftType::Mixed)
        {
            controller.precompute_sudden_drifts();
        }

        controller
    }

    /// Pre-compute which periods will have sudden drift events.
    fn precompute_sudden_drifts(&mut self) {
        for period in 0..self.total_periods {
            if period >= self.config.drift_start_period
                && self.rng.gen::<f64>() < self.config.sudden_drift_probability
            {
                self.sudden_drift_periods.push(period);
            }
        }
    }

    /// Check if drift is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Compute drift adjustments for a specific period (0-indexed).
    pub fn compute_adjustments(&self, period: u32) -> DriftAdjustments {
        if !self.config.enabled {
            return DriftAdjustments::none();
        }

        // No drift before start period
        if period < self.config.drift_start_period {
            return DriftAdjustments::none();
        }

        let effective_period = period - self.config.drift_start_period;
        let mut adjustments = DriftAdjustments::none();

        match self.config.drift_type {
            DriftType::Gradual => {
                self.apply_gradual_drift(&mut adjustments, effective_period);
            }
            DriftType::Sudden => {
                self.apply_sudden_drift(&mut adjustments, period);
            }
            DriftType::Recurring => {
                self.apply_recurring_drift(&mut adjustments, effective_period);
            }
            DriftType::Mixed => {
                // Combine gradual background drift with sudden events
                self.apply_gradual_drift(&mut adjustments, effective_period);
                self.apply_sudden_drift(&mut adjustments, period);
            }
        }

        // Apply seasonal drift if enabled (additive to other drift)
        if self.config.seasonal_drift {
            adjustments.seasonal_factor = self.compute_seasonal_factor(period);
        }

        adjustments
    }

    /// Apply gradual drift (compound growth model).
    fn apply_gradual_drift(&self, adjustments: &mut DriftAdjustments, effective_period: u32) {
        let p = effective_period as f64;

        // Compound growth: (1 + rate)^period
        adjustments.amount_mean_multiplier = (1.0 + self.config.amount_mean_drift).powf(p);

        adjustments.amount_variance_multiplier = (1.0 + self.config.amount_variance_drift).powf(p);

        // Linear accumulation for anomaly rate
        adjustments.anomaly_rate_adjustment = self.config.anomaly_rate_drift * p;

        // Concept drift accumulates but is bounded 0-1
        adjustments.concept_drift_factor = (self.config.concept_drift_rate * p).min(1.0);
    }

    /// Apply sudden drift based on pre-computed events.
    fn apply_sudden_drift(&self, adjustments: &mut DriftAdjustments, period: u32) {
        // Count how many sudden events have occurred up to this period
        let events_occurred: usize = self
            .sudden_drift_periods
            .iter()
            .filter(|&&p| p <= period)
            .count();

        if events_occurred > 0 {
            adjustments.sudden_drift_occurred = self.sudden_drift_periods.contains(&period);

            // Each sudden event multiplies by the magnitude
            let cumulative_magnitude = self
                .config
                .sudden_drift_magnitude
                .powi(events_occurred as i32);

            adjustments.amount_mean_multiplier *= cumulative_magnitude;
            adjustments.amount_variance_multiplier *= cumulative_magnitude.sqrt();
            // Variance grows slower
        }
    }

    /// Apply recurring (seasonal) drift patterns.
    fn apply_recurring_drift(&self, adjustments: &mut DriftAdjustments, effective_period: u32) {
        // 12-month cycle for seasonality
        let cycle_position = (effective_period % 12) as f64;
        let cycle_radians = (cycle_position / 12.0) * 2.0 * std::f64::consts::PI;

        // Sinusoidal pattern with configurable amplitude
        let seasonal_amplitude = self.config.concept_drift_rate;
        adjustments.amount_mean_multiplier = 1.0 + seasonal_amplitude * cycle_radians.sin();

        // Phase-shifted variance pattern
        adjustments.amount_variance_multiplier =
            1.0 + (seasonal_amplitude * 0.5) * (cycle_radians + std::f64::consts::FRAC_PI_2).sin();
    }

    /// Compute seasonal factor based on period (month).
    fn compute_seasonal_factor(&self, period: u32) -> f64 {
        // Map period to month (0-11)
        let month = period % 12;

        // Q4 spike (Oct-Dec), Q1 dip (Jan-Feb)
        match month {
            0 | 1 => 0.85, // Jan-Feb: post-holiday slowdown
            2 => 0.90,     // Mar: recovering
            3 | 4 => 0.95, // Apr-May: Q2 start
            5 => 1.0,      // Jun: mid-year
            6 | 7 => 0.95, // Jul-Aug: summer slowdown
            8 => 1.0,      // Sep: back to business
            9 => 1.10,     // Oct: Q4 ramp-up
            10 => 1.20,    // Nov: pre-holiday surge
            11 => 1.30,    // Dec: year-end close
            _ => 1.0,
        }
    }

    /// Get the list of periods with sudden drift events.
    pub fn sudden_drift_periods(&self) -> &[u32] {
        &self.sudden_drift_periods
    }

    /// Get the configuration.
    pub fn config(&self) -> &DriftConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_drift_when_disabled() {
        let config = DriftConfig::default();
        let controller = DriftController::new(config, 42, 12);

        let adjustments = controller.compute_adjustments(6);
        assert!(!controller.is_enabled());
        assert!((adjustments.amount_mean_multiplier - 1.0).abs() < 0.001);
        assert!((adjustments.anomaly_rate_adjustment).abs() < 0.001);
    }

    #[test]
    fn test_gradual_drift() {
        let config = DriftConfig {
            enabled: true,
            amount_mean_drift: 0.02,
            anomaly_rate_drift: 0.001,
            drift_type: DriftType::Gradual,
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // Period 0: no drift yet
        let adj0 = controller.compute_adjustments(0);
        assert!((adj0.amount_mean_multiplier - 1.0).abs() < 0.001);

        // Period 6: ~12.6% drift (1.02^6 ≈ 1.126)
        let adj6 = controller.compute_adjustments(6);
        assert!(adj6.amount_mean_multiplier > 1.10);
        assert!(adj6.amount_mean_multiplier < 1.15);

        // Period 12: ~26.8% drift (1.02^12 ≈ 1.268)
        let adj12 = controller.compute_adjustments(12);
        assert!(adj12.amount_mean_multiplier > 1.20);
        assert!(adj12.amount_mean_multiplier < 1.30);
    }

    #[test]
    fn test_drift_start_period() {
        let config = DriftConfig {
            enabled: true,
            amount_mean_drift: 0.02,
            drift_start_period: 3,
            drift_type: DriftType::Gradual,
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // Before drift start: no drift
        let adj2 = controller.compute_adjustments(2);
        assert!((adj2.amount_mean_multiplier - 1.0).abs() < 0.001);

        // At drift start: no drift yet (effective_period = 0)
        let adj3 = controller.compute_adjustments(3);
        assert!((adj3.amount_mean_multiplier - 1.0).abs() < 0.001);

        // After drift start: drift begins
        let adj6 = controller.compute_adjustments(6);
        assert!(adj6.amount_mean_multiplier > 1.0);
    }

    #[test]
    fn test_seasonal_factor() {
        let config = DriftConfig {
            enabled: true,
            seasonal_drift: true,
            drift_type: DriftType::Gradual,
            ..Default::default()
        };
        let controller = DriftController::new(config, 42, 12);

        // December (month 11) should have highest seasonal factor
        let adj_dec = controller.compute_adjustments(11);
        assert!(adj_dec.seasonal_factor > 1.2);

        // January (month 0) should have lower seasonal factor
        let adj_jan = controller.compute_adjustments(0);
        assert!(adj_jan.seasonal_factor < 0.9);
    }

    #[test]
    fn test_sudden_drift_reproducibility() {
        let config = DriftConfig {
            enabled: true,
            sudden_drift_probability: 0.5,
            sudden_drift_magnitude: 1.5,
            drift_type: DriftType::Sudden,
            ..Default::default()
        };

        // Same seed should produce same sudden drift periods
        let controller1 = DriftController::new(config.clone(), 42, 12);
        let controller2 = DriftController::new(config, 42, 12);

        assert_eq!(
            controller1.sudden_drift_periods(),
            controller2.sudden_drift_periods()
        );
    }
}
