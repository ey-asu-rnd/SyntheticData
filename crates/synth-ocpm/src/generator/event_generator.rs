//! Core OCPM event generator.
//!
//! Generates OCPM events from document flows and business processes.

use chrono::{DateTime, Duration, Utc};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use uuid::Uuid;

use crate::models::{
    ActivityType, CaseTrace, EventLifecycle, EventObjectRef, ObjectAttributeValue, ObjectInstance,
    ObjectQualifier, ObjectRelationship, ObjectType, OcpmEvent,
};
use synth_core::models::BusinessProcess;

/// Configuration for OCPM event generation.
#[derive(Debug, Clone)]
pub struct OcpmGeneratorConfig {
    /// Enable P2P process events
    pub generate_p2p: bool,
    /// Enable O2C process events
    pub generate_o2c: bool,
    /// Rate of happy path (normal) variants
    pub happy_path_rate: f64,
    /// Rate of exception path variants
    pub exception_path_rate: f64,
    /// Rate of error path variants
    pub error_path_rate: f64,
    /// Add duration variability to events
    pub add_duration_variability: bool,
    /// Standard deviation factor for duration
    pub duration_std_dev_factor: f64,
}

impl Default for OcpmGeneratorConfig {
    fn default() -> Self {
        Self {
            generate_p2p: true,
            generate_o2c: true,
            happy_path_rate: 0.75,
            exception_path_rate: 0.20,
            error_path_rate: 0.05,
            add_duration_variability: true,
            duration_std_dev_factor: 0.3,
        }
    }
}

/// Main OCPM event generator.
pub struct OcpmEventGenerator {
    /// Random number generator
    rng: ChaCha8Rng,
    /// Configuration
    config: OcpmGeneratorConfig,
    /// P2P activity types
    p2p_activities: Vec<ActivityType>,
    /// O2C activity types
    o2c_activities: Vec<ActivityType>,
    /// Case ID counter for generating unique case IDs
    case_counter: u64,
}

impl OcpmEventGenerator {
    /// Create a new OCPM event generator with a seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config: OcpmGeneratorConfig::default(),
            p2p_activities: ActivityType::p2p_activities(),
            o2c_activities: ActivityType::o2c_activities(),
            case_counter: 0,
        }
    }

    /// Create with custom configuration.
    pub fn with_config(seed: u64, config: OcpmGeneratorConfig) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
            config,
            p2p_activities: ActivityType::p2p_activities(),
            o2c_activities: ActivityType::o2c_activities(),
            case_counter: 0,
        }
    }

    /// Generate a new case ID.
    pub fn new_case_id(&mut self) -> Uuid {
        self.case_counter += 1;
        Uuid::new_v4()
    }

    /// Select a process variant type based on configuration.
    pub fn select_variant_type(&mut self) -> VariantType {
        let r: f64 = self.rng.gen();
        if r < self.config.happy_path_rate {
            VariantType::HappyPath
        } else if r < self.config.happy_path_rate + self.config.exception_path_rate {
            VariantType::ExceptionPath
        } else {
            VariantType::ErrorPath
        }
    }

    /// Calculate event timestamp with variability.
    pub fn calculate_event_time(
        &mut self,
        base_time: DateTime<Utc>,
        activity: &ActivityType,
    ) -> DateTime<Utc> {
        if let Some(typical_minutes) = activity.typical_duration_minutes {
            let std_dev = activity.duration_std_dev.unwrap_or(typical_minutes * 0.3);

            if self.config.add_duration_variability {
                // Add some variability using normal-like distribution
                let variability: f64 = self.rng.gen_range(-2.0..2.0) * std_dev;
                let actual_minutes = (typical_minutes + variability).max(1.0);
                base_time + Duration::minutes(actual_minutes as i64)
            } else {
                base_time + Duration::minutes(typical_minutes as i64)
            }
        } else {
            base_time + Duration::minutes(5) // Default 5 minutes
        }
    }

    /// Create an event from an activity type.
    pub fn create_event(
        &mut self,
        activity: &ActivityType,
        timestamp: DateTime<Utc>,
        resource_id: &str,
        company_code: &str,
        case_id: Uuid,
    ) -> OcpmEvent {
        OcpmEvent::new(
            &activity.activity_id,
            &activity.name,
            timestamp,
            resource_id,
            company_code,
        )
        .with_case(case_id)
        .with_lifecycle(if activity.is_automated {
            EventLifecycle::Atomic
        } else {
            EventLifecycle::Complete
        })
    }

    /// Create an object instance for a document.
    pub fn create_object(
        &self,
        object_type: &ObjectType,
        external_id: &str,
        company_code: &str,
        created_at: DateTime<Utc>,
    ) -> ObjectInstance {
        ObjectInstance::new(&object_type.type_id, external_id, company_code)
            .with_state("active")
            .with_created_at(created_at)
    }

    /// Create object reference for an event.
    pub fn create_object_ref(
        &self,
        object: &ObjectInstance,
        qualifier: ObjectQualifier,
    ) -> EventObjectRef {
        EventObjectRef::new(object.object_id, &object.object_type_id, qualifier)
            .with_external_id(&object.external_id)
    }

    /// Add an attribute to an event.
    pub fn add_event_attribute(event: &mut OcpmEvent, key: &str, value: ObjectAttributeValue) {
        event.attributes.insert(key.into(), value);
    }

    /// Generate a complete case trace from events.
    pub fn create_case_trace(
        &self,
        _case_id: Uuid,
        events: &[OcpmEvent],
        business_process: BusinessProcess,
        primary_object_id: Uuid,
        primary_object_type: &str,
        company_code: &str,
    ) -> CaseTrace {
        let activity_sequence: Vec<String> = events.iter().map(|e| e.activity_id.clone()).collect();

        let start_time = events.first().map(|e| e.timestamp).unwrap_or_else(Utc::now);
        let end_time = events.last().map(|e| e.timestamp);

        let mut trace = CaseTrace::new(
            business_process,
            primary_object_id,
            primary_object_type,
            company_code,
        );
        trace.activity_sequence = activity_sequence;
        trace.event_ids = events.iter().map(|e| e.event_id).collect();
        trace.start_time = start_time;
        trace.end_time = end_time;
        trace
    }

    /// Select a resource for an activity.
    pub fn select_resource(
        &mut self,
        activity: &ActivityType,
        available_users: &[String],
    ) -> String {
        if activity.is_automated {
            "SYSTEM".into()
        } else if available_users.is_empty() {
            format!("USER{:04}", self.rng.gen_range(1..100))
        } else {
            let idx = self.rng.gen_range(0..available_users.len());
            available_users[idx].clone()
        }
    }

    /// Get P2P activities.
    pub fn p2p_activities(&self) -> &[ActivityType] {
        &self.p2p_activities
    }

    /// Get O2C activities.
    pub fn o2c_activities(&self) -> &[ActivityType] {
        &self.o2c_activities
    }

    /// Generate random delay between activities (in minutes).
    pub fn generate_inter_activity_delay(
        &mut self,
        min_minutes: i64,
        max_minutes: i64,
    ) -> Duration {
        let minutes = self.rng.gen_range(min_minutes..=max_minutes);
        Duration::minutes(minutes)
    }

    /// Check if an activity should be skipped (for exception paths).
    pub fn should_skip_activity(&mut self, skip_probability: f64) -> bool {
        self.rng.gen::<f64>() < skip_probability
    }

    /// Generate a random boolean with given probability.
    pub fn random_bool(&mut self, probability: f64) -> bool {
        self.rng.gen::<f64>() < probability
    }
}

/// Type of process variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantType {
    /// Normal/happy path - all activities completed successfully
    HappyPath,
    /// Exception path - some activities skipped or modified
    ExceptionPath,
    /// Error path - process aborted or failed
    ErrorPath,
}

impl VariantType {
    /// Get a description of this variant type.
    pub fn description(&self) -> &'static str {
        match self {
            Self::HappyPath => "Standard process execution",
            Self::ExceptionPath => "Process with exceptions or variations",
            Self::ErrorPath => "Process failed or aborted",
        }
    }
}

/// Result of generating events for a case.
#[derive(Debug)]
pub struct CaseGenerationResult {
    /// Generated events
    pub events: Vec<OcpmEvent>,
    /// Generated objects
    pub objects: Vec<ObjectInstance>,
    /// Generated relationships
    pub relationships: Vec<ObjectRelationship>,
    /// Case trace
    pub case_trace: CaseTrace,
    /// Variant type used
    pub variant_type: VariantType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = OcpmEventGenerator::new(42);
        assert!(!generator.p2p_activities.is_empty());
        assert!(!generator.o2c_activities.is_empty());
    }

    #[test]
    fn test_variant_selection() {
        let mut generator = OcpmEventGenerator::new(42);

        // Generate many variants and check distribution
        let mut happy = 0;
        let mut exception = 0;
        let mut error = 0;

        for _ in 0..1000 {
            match generator.select_variant_type() {
                VariantType::HappyPath => happy += 1,
                VariantType::ExceptionPath => exception += 1,
                VariantType::ErrorPath => error += 1,
            }
        }

        // Should be roughly 75%/20%/5%
        assert!(happy > 600 && happy < 850);
        assert!(exception > 100 && exception < 300);
        assert!(error > 10 && error < 100);
    }

    #[test]
    fn test_case_id_generation() {
        let mut generator = OcpmEventGenerator::new(42);
        let id1 = generator.new_case_id();
        let id2 = generator.new_case_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_event_creation() {
        let mut generator = OcpmEventGenerator::new(42);
        let activity = ActivityType::create_po();
        let case_id = generator.new_case_id();

        let event = generator.create_event(&activity, Utc::now(), "user001", "1000", case_id);

        assert_eq!(event.activity_id, "create_po");
        assert_eq!(event.case_id, Some(case_id));
    }
}
