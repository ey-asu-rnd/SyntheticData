//! Distribution fitting utilities.

use crate::models::{DistributionParams, DistributionType, NumericStats};

/// Fit a distribution to observed statistics.
pub fn fit_to_stats(stats: &NumericStats) -> (DistributionType, DistributionParams) {
    // If already fitted, return existing
    if stats.distribution != DistributionType::Unknown {
        return (stats.distribution, stats.distribution_params.clone());
    }

    // Attempt to fit based on characteristics
    let mean = stats.mean;
    let std_dev = stats.std_dev;
    let min = stats.min;
    let max = stats.max;

    // Check for uniform distribution
    let range = max - min;
    if range > 0.0 {
        let expected_std_uniform = range / (12.0_f64).sqrt();
        if (std_dev - expected_std_uniform).abs() / expected_std_uniform < 0.15 {
            return (
                DistributionType::Uniform,
                DistributionParams::uniform(min, max),
            );
        }
    }

    // Check for exponential (mean ≈ std_dev for exponential)
    if mean > 0.0 && min >= 0.0 && (std_dev / mean - 1.0).abs() < 0.2 {
        return (
            DistributionType::Exponential,
            DistributionParams::exponential(1.0 / mean),
        );
    }

    // For positive data, prefer log-normal
    if min > 0.0 {
        let log_values_mean = mean.ln();
        let cv = std_dev / mean; // Coefficient of variation
        let sigma = (1.0 + cv.powi(2)).ln().sqrt();
        let mu = log_values_mean - sigma.powi(2) / 2.0;

        return (
            DistributionType::LogNormal,
            DistributionParams::log_normal(mu, sigma),
        );
    }

    // Default to normal
    (
        DistributionType::Normal,
        DistributionParams::normal(mean, std_dev),
    )
}

/// Estimate log-normal parameters using method of moments.
pub fn estimate_lognormal_params(mean: f64, variance: f64) -> (f64, f64) {
    // mu = ln(mean^2 / sqrt(variance + mean^2))
    // sigma^2 = ln(1 + variance / mean^2)
    if mean <= 0.0 {
        return (0.0, 1.0);
    }

    let sigma_sq = (1.0 + variance / mean.powi(2)).ln();
    let mu = mean.ln() - sigma_sq / 2.0;

    (mu, sigma_sq.sqrt())
}

/// Estimate normal parameters (trivial).
pub fn estimate_normal_params(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 1.0);
    }

    let n = values.len() as f64;
    let mean: f64 = values.iter().sum::<f64>() / n;
    let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;

    (mean, variance.sqrt())
}

/// Goodness of fit test (simplified KS-like).
pub fn goodness_of_fit(observed: &[f64], dist_type: DistributionType, params: &DistributionParams) -> f64 {
    // Returns a score 0-1 where 1 is perfect fit
    if observed.is_empty() {
        return 0.0;
    }

    let mut sorted = observed.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = sorted.len();
    let mut max_diff = 0.0;

    for (i, &x) in sorted.iter().enumerate() {
        let empirical_cdf = (i + 1) as f64 / n as f64;
        let theoretical_cdf = theoretical_cdf(x, dist_type, params);
        let diff = (empirical_cdf - theoretical_cdf).abs();
        if diff > max_diff {
            max_diff = diff;
        }
    }

    // Convert to 0-1 score (lower KS statistic = better fit)
    1.0 - max_diff.min(1.0)
}

/// Theoretical CDF for a distribution.
fn theoretical_cdf(x: f64, dist_type: DistributionType, params: &DistributionParams) -> f64 {
    match dist_type {
        DistributionType::Normal => {
            let mean = params.param1.unwrap_or(0.0);
            let std_dev = params.param2.unwrap_or(1.0);
            normal_cdf(x, mean, std_dev)
        }
        DistributionType::LogNormal => {
            if x <= 0.0 {
                return 0.0;
            }
            let mu = params.param1.unwrap_or(0.0);
            let sigma = params.param2.unwrap_or(1.0);
            normal_cdf(x.ln(), mu, sigma)
        }
        DistributionType::Uniform => {
            let a = params.param1.unwrap_or(0.0);
            let b = params.param2.unwrap_or(1.0);
            if x < a {
                0.0
            } else if x > b {
                1.0
            } else {
                (x - a) / (b - a)
            }
        }
        DistributionType::Exponential => {
            let rate = params.param1.unwrap_or(1.0);
            if x < 0.0 {
                0.0
            } else {
                1.0 - (-rate * x).exp()
            }
        }
        _ => 0.5, // Placeholder for other distributions
    }
}

/// Normal CDF approximation.
fn normal_cdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev == 0.0 {
        return if x >= mean { 1.0 } else { 0.0 };
    }

    let z = (x - mean) / std_dev;
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

/// Error function approximation.
fn erf(x: f64) -> f64 {
    // Approximation from Abramowitz and Stegun
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_lognormal() {
        let (mu, sigma) = estimate_lognormal_params(100.0, 2500.0);
        assert!(mu > 0.0);
        assert!(sigma > 0.0);
    }

    #[test]
    fn test_normal_cdf() {
        assert!((normal_cdf(0.0, 0.0, 1.0) - 0.5).abs() < 0.01);
        assert!(normal_cdf(3.0, 0.0, 1.0) > 0.99);
        assert!(normal_cdf(-3.0, 0.0, 1.0) < 0.01);
    }
}
