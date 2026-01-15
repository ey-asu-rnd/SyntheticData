//! Anomaly patterns for realistic distribution.
//!
//! Patterns control how anomalies are distributed across time and entities,
//! including clustering behavior and temporal patterns.

use chrono::{Datelike, NaiveDate, Weekday};
use rand::Rng;
use std::collections::HashMap;

/// Temporal pattern for anomaly injection.
#[derive(Debug, Clone)]
pub enum TemporalPattern {
    /// Uniform distribution across all periods.
    Uniform,
    /// Higher probability at period/year end.
    PeriodEndSpike {
        /// Multiplier for month-end days.
        month_end_multiplier: f64,
        /// Multiplier for quarter-end.
        quarter_end_multiplier: f64,
        /// Multiplier for year-end.
        year_end_multiplier: f64,
    },
    /// Higher probability at specific times.
    TimeBased {
        /// Multiplier for after-hours.
        after_hours_multiplier: f64,
        /// Multiplier for weekends.
        weekend_multiplier: f64,
    },
    /// Seasonal pattern.
    Seasonal {
        /// Multipliers by month (1-12).
        month_multipliers: [f64; 12],
    },
    /// Custom pattern function.
    Custom {
        /// Name of the pattern.
        name: String,
    },
}

impl Default for TemporalPattern {
    fn default() -> Self {
        TemporalPattern::PeriodEndSpike {
            month_end_multiplier: 2.0,
            quarter_end_multiplier: 3.0,
            year_end_multiplier: 5.0,
        }
    }
}

impl TemporalPattern {
    /// Calculates the probability multiplier for a given date.
    pub fn probability_multiplier(&self, date: NaiveDate) -> f64 {
        match self {
            TemporalPattern::Uniform => 1.0,
            TemporalPattern::PeriodEndSpike {
                month_end_multiplier,
                quarter_end_multiplier,
                year_end_multiplier,
            } => {
                let day = date.day();
                let month = date.month();

                // Year end (December 28-31)
                if month == 12 && day >= 28 {
                    return *year_end_multiplier;
                }

                // Quarter end (Mar, Jun, Sep, Dec last 3 days)
                if matches!(month, 3 | 6 | 9 | 12) && day >= 28 {
                    return *quarter_end_multiplier;
                }

                // Month end (last 3 days)
                if day >= 28 {
                    return *month_end_multiplier;
                }

                1.0
            }
            TemporalPattern::TimeBased {
                after_hours_multiplier,
                weekend_multiplier,
            } => {
                let weekday = date.weekday();
                if weekday == Weekday::Sat || weekday == Weekday::Sun {
                    return *weekend_multiplier;
                }
                // Assume all entries have potential for after-hours
                // In practice, this would check timestamp
                1.0
            }
            TemporalPattern::Seasonal { month_multipliers } => {
                let month_idx = (date.month() - 1) as usize;
                month_multipliers[month_idx]
            }
            TemporalPattern::Custom { .. } => 1.0,
        }
    }

    /// Creates a standard audit season pattern (higher in Q1).
    pub fn audit_season() -> Self {
        TemporalPattern::Seasonal {
            month_multipliers: [
                2.0, 2.0, 1.5, // Q1 - audit busy season
                1.0, 1.0, 1.2, // Q2 - quarter end
                1.0, 1.0, 1.2, // Q3 - quarter end
                1.0, 1.0, 3.0, // Q4 - year end
            ],
        }
    }
}

/// Clustering behavior for anomalies.
#[derive(Debug, Clone)]
pub struct ClusteringConfig {
    /// Whether clustering is enabled.
    pub enabled: bool,
    /// Probability that an anomaly starts a new cluster.
    pub cluster_start_probability: f64,
    /// Probability that next anomaly joins current cluster.
    pub cluster_continuation_probability: f64,
    /// Minimum cluster size.
    pub min_cluster_size: usize,
    /// Maximum cluster size.
    pub max_cluster_size: usize,
    /// Time window for cluster (days).
    pub cluster_time_window_days: i64,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cluster_start_probability: 0.3,
            cluster_continuation_probability: 0.7,
            min_cluster_size: 2,
            max_cluster_size: 10,
            cluster_time_window_days: 7,
        }
    }
}

/// Manages anomaly clustering.
pub struct ClusterManager {
    config: ClusteringConfig,
    /// Current active cluster ID.
    current_cluster: Option<String>,
    /// Count of anomalies in current cluster.
    current_cluster_size: usize,
    /// Start date of current cluster.
    cluster_start_date: Option<NaiveDate>,
    /// Next cluster ID to assign.
    next_cluster_id: u64,
    /// Cluster statistics.
    cluster_stats: HashMap<String, ClusterStats>,
}

/// Statistics for a cluster.
#[derive(Debug, Clone, Default)]
pub struct ClusterStats {
    /// Number of anomalies in cluster.
    pub size: usize,
    /// Start date.
    pub start_date: Option<NaiveDate>,
    /// End date.
    pub end_date: Option<NaiveDate>,
    /// Anomaly types in cluster.
    pub anomaly_types: Vec<String>,
}

impl ClusterManager {
    /// Creates a new cluster manager.
    pub fn new(config: ClusteringConfig) -> Self {
        Self {
            config,
            current_cluster: None,
            current_cluster_size: 0,
            cluster_start_date: None,
            next_cluster_id: 1,
            cluster_stats: HashMap::new(),
        }
    }

    /// Determines the cluster ID for a new anomaly.
    pub fn assign_cluster<R: Rng>(
        &mut self,
        date: NaiveDate,
        anomaly_type: &str,
        rng: &mut R,
    ) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        // Check if we should continue current cluster
        if let Some(ref cluster_id) = self.current_cluster {
            if let Some(start) = self.cluster_start_date {
                let days_elapsed = (date - start).num_days();

                // Check if within time window and not at max size
                if days_elapsed <= self.config.cluster_time_window_days
                    && self.current_cluster_size < self.config.max_cluster_size
                    && rng.gen::<f64>() < self.config.cluster_continuation_probability
                {
                    self.current_cluster_size += 1;

                    // Update cluster stats
                    if let Some(stats) = self.cluster_stats.get_mut(cluster_id) {
                        stats.size += 1;
                        stats.end_date = Some(date);
                        stats.anomaly_types.push(anomaly_type.to_string());
                    }

                    return Some(cluster_id.clone());
                }
            }

            // End current cluster if at min size
            if self.current_cluster_size >= self.config.min_cluster_size {
                self.current_cluster = None;
            }
        }

        // Decide whether to start a new cluster
        if rng.gen::<f64>() < self.config.cluster_start_probability {
            let cluster_id = format!("CLU{:06}", self.next_cluster_id);
            self.next_cluster_id += 1;

            self.current_cluster = Some(cluster_id.clone());
            self.current_cluster_size = 1;
            self.cluster_start_date = Some(date);

            // Initialize cluster stats
            self.cluster_stats.insert(
                cluster_id.clone(),
                ClusterStats {
                    size: 1,
                    start_date: Some(date),
                    end_date: Some(date),
                    anomaly_types: vec![anomaly_type.to_string()],
                },
            );

            return Some(cluster_id);
        }

        None
    }

    /// Gets cluster statistics.
    pub fn get_cluster_stats(&self, cluster_id: &str) -> Option<&ClusterStats> {
        self.cluster_stats.get(cluster_id)
    }

    /// Gets all cluster statistics.
    pub fn all_cluster_stats(&self) -> &HashMap<String, ClusterStats> {
        &self.cluster_stats
    }

    /// Returns the number of clusters created.
    pub fn cluster_count(&self) -> usize {
        self.cluster_stats.len()
    }
}

/// Entity targeting pattern.
#[derive(Debug, Clone)]
pub enum EntityTargetingPattern {
    /// Random entity selection.
    Random,
    /// Weighted by transaction volume.
    VolumeWeighted,
    /// Focus on specific entity types.
    TypeFocused {
        /// Target entity types with weights.
        type_weights: HashMap<String, f64>,
    },
    /// Repeat offender pattern (same entities).
    RepeatOffender {
        /// Probability of targeting same entity.
        repeat_probability: f64,
    },
}

impl Default for EntityTargetingPattern {
    fn default() -> Self {
        EntityTargetingPattern::Random
    }
}

/// Manages entity targeting for anomalies.
pub struct EntityTargetingManager {
    pattern: EntityTargetingPattern,
    /// Recently targeted entities.
    recent_targets: Vec<String>,
    /// Maximum recent targets to track.
    max_recent: usize,
    /// Entity hit counts.
    hit_counts: HashMap<String, usize>,
}

impl EntityTargetingManager {
    /// Creates a new entity targeting manager.
    pub fn new(pattern: EntityTargetingPattern) -> Self {
        Self {
            pattern,
            recent_targets: Vec::new(),
            max_recent: 20,
            hit_counts: HashMap::new(),
        }
    }

    /// Selects an entity to target.
    pub fn select_entity<R: Rng>(&mut self, candidates: &[String], rng: &mut R) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }

        let selected = match &self.pattern {
            EntityTargetingPattern::Random => {
                candidates[rng.gen_range(0..candidates.len())].clone()
            }
            EntityTargetingPattern::VolumeWeighted => {
                // In practice, would weight by actual volume
                // For now, use random
                candidates[rng.gen_range(0..candidates.len())].clone()
            }
            EntityTargetingPattern::TypeFocused { type_weights } => {
                // Filter by type weights
                let weighted: Vec<_> = candidates
                    .iter()
                    .filter_map(|c| {
                        type_weights.get(c).map(|&w| (c.clone(), w))
                    })
                    .collect();

                if weighted.is_empty() {
                    candidates[rng.gen_range(0..candidates.len())].clone()
                } else {
                    let total: f64 = weighted.iter().map(|(_, w)| w).sum();
                    let mut r = rng.gen::<f64>() * total;
                    for (entity, weight) in &weighted {
                        r -= weight;
                        if r <= 0.0 {
                            return Some(entity.clone());
                        }
                    }
                    weighted[0].0.clone()
                }
            }
            EntityTargetingPattern::RepeatOffender { repeat_probability } => {
                // Check if we should repeat a recent target
                if !self.recent_targets.is_empty() && rng.gen::<f64>() < *repeat_probability {
                    let idx = rng.gen_range(0..self.recent_targets.len());
                    self.recent_targets[idx].clone()
                } else {
                    candidates[rng.gen_range(0..candidates.len())].clone()
                }
            }
        };

        // Track the selection
        self.recent_targets.push(selected.clone());
        if self.recent_targets.len() > self.max_recent {
            self.recent_targets.remove(0);
        }

        *self.hit_counts.entry(selected.clone()).or_insert(0) += 1;

        Some(selected)
    }

    /// Gets hit count for an entity.
    pub fn hit_count(&self, entity: &str) -> usize {
        *self.hit_counts.get(entity).unwrap_or(&0)
    }
}

/// Combined pattern configuration.
#[derive(Debug, Clone)]
pub struct AnomalyPatternConfig {
    /// Temporal pattern.
    pub temporal_pattern: TemporalPattern,
    /// Clustering configuration.
    pub clustering: ClusteringConfig,
    /// Entity targeting pattern.
    pub entity_targeting: EntityTargetingPattern,
    /// Whether to inject anomalies in batches.
    pub batch_injection: bool,
    /// Batch size range.
    pub batch_size_range: (usize, usize),
}

impl Default for AnomalyPatternConfig {
    fn default() -> Self {
        Self {
            temporal_pattern: TemporalPattern::default(),
            clustering: ClusteringConfig::default(),
            entity_targeting: EntityTargetingPattern::default(),
            batch_injection: false,
            batch_size_range: (2, 5),
        }
    }
}

/// Determines if an anomaly should be injected at this point.
pub fn should_inject_anomaly<R: Rng>(
    base_rate: f64,
    date: NaiveDate,
    pattern: &TemporalPattern,
    rng: &mut R,
) -> bool {
    let multiplier = pattern.probability_multiplier(date);
    let adjusted_rate = (base_rate * multiplier).min(1.0);
    rng.gen::<f64>() < adjusted_rate
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_temporal_pattern_multiplier() {
        let pattern = TemporalPattern::default();

        // Regular day
        let regular = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        assert_eq!(pattern.probability_multiplier(regular), 1.0);

        // Month end
        let month_end = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
        assert!(pattern.probability_multiplier(month_end) > 1.0);

        // Year end
        let year_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        assert!(pattern.probability_multiplier(year_end) > pattern.probability_multiplier(month_end));
    }

    #[test]
    fn test_cluster_manager() {
        let mut manager = ClusterManager::new(ClusteringConfig::default());
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();

        // Generate several anomalies and check clustering
        let mut clustered = 0;
        for i in 0..20 {
            let d = date + chrono::Duration::days(i % 7); // Within time window
            if manager.assign_cluster(d, "TestType", &mut rng).is_some() {
                clustered += 1;
            }
        }

        // Some should be clustered
        assert!(clustered > 0);
        assert!(manager.cluster_count() > 0);
    }

    #[test]
    fn test_should_inject_anomaly() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let pattern = TemporalPattern::default();

        let regular_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let year_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        // Count injections over many trials
        let mut regular_count = 0;
        let mut year_end_count = 0;

        for _ in 0..1000 {
            if should_inject_anomaly(0.1, regular_date, &pattern, &mut rng) {
                regular_count += 1;
            }
            if should_inject_anomaly(0.1, year_end, &pattern, &mut rng) {
                year_end_count += 1;
            }
        }

        // Year end should have more injections due to multiplier
        assert!(year_end_count > regular_count);
    }
}
