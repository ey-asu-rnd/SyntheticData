//! User persona and behavior models.
//!
//! Defines user personas and behavioral patterns for realistic
//! transaction generation, including working hours, error rates,
//! and transaction volumes.

use serde::{Deserialize, Serialize};

/// User persona classification for behavioral modeling.
///
/// Different personas exhibit different transaction patterns, timing,
/// error rates, and access to accounts/functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UserPersona {
    /// Entry-level accountant with limited access
    JuniorAccountant,
    /// Experienced accountant with broader access
    SeniorAccountant,
    /// Financial controller with approval authority
    Controller,
    /// Management with override capabilities
    Manager,
    /// CFO/Finance Director with full access
    Executive,
    /// Automated batch job or interface
    #[default]
    AutomatedSystem,
    /// External auditor with read access
    ExternalAuditor,
    /// Fraud actor for simulation scenarios
    FraudActor,
}

impl UserPersona {
    /// Check if this persona represents a human user.
    pub fn is_human(&self) -> bool {
        !matches!(self, Self::AutomatedSystem)
    }

    /// Check if this persona has approval authority.
    pub fn has_approval_authority(&self) -> bool {
        matches!(self, Self::Controller | Self::Manager | Self::Executive)
    }

    /// Get typical error rate for this persona (0.0-1.0).
    pub fn error_rate(&self) -> f64 {
        match self {
            Self::JuniorAccountant => 0.02,
            Self::SeniorAccountant => 0.005,
            Self::Controller => 0.002,
            Self::Manager => 0.003,
            Self::Executive => 0.001,
            Self::AutomatedSystem => 0.0001,
            Self::ExternalAuditor => 0.0,
            Self::FraudActor => 0.01,
        }
    }

    /// Get typical transaction volume per day.
    pub fn typical_daily_volume(&self) -> (u32, u32) {
        match self {
            Self::JuniorAccountant => (20, 100),
            Self::SeniorAccountant => (10, 50),
            Self::Controller => (5, 20),
            Self::Manager => (1, 10),
            Self::Executive => (0, 5),
            Self::AutomatedSystem => (100, 10000),
            Self::ExternalAuditor => (0, 0),
            Self::FraudActor => (1, 5),
        }
    }

    /// Get approval threshold amount.
    pub fn approval_threshold(&self) -> Option<f64> {
        match self {
            Self::JuniorAccountant => Some(1000.0),
            Self::SeniorAccountant => Some(10000.0),
            Self::Controller => Some(100000.0),
            Self::Manager => Some(500000.0),
            Self::Executive => None, // Unlimited
            Self::AutomatedSystem => Some(1000000.0),
            Self::ExternalAuditor => Some(0.0), // Read-only
            Self::FraudActor => Some(10000.0),
        }
    }
}

/// Working hours pattern for human users.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHoursPattern {
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Peak hours (typically mid-morning and mid-afternoon)
    pub peak_hours: Vec<u8>,
    /// Probability of weekend work
    pub weekend_probability: f64,
    /// Probability of after-hours work
    pub after_hours_probability: f64,
}

impl Default for WorkingHoursPattern {
    fn default() -> Self {
        Self {
            start_hour: 8,
            end_hour: 18,
            peak_hours: vec![10, 11, 14, 15],
            weekend_probability: 0.05,
            after_hours_probability: 0.10,
        }
    }
}

impl WorkingHoursPattern {
    /// Pattern for European office hours.
    pub fn european() -> Self {
        Self {
            start_hour: 9,
            end_hour: 17,
            peak_hours: vec![10, 11, 14, 15],
            weekend_probability: 0.02,
            after_hours_probability: 0.05,
        }
    }

    /// Pattern for US office hours.
    pub fn us_standard() -> Self {
        Self {
            start_hour: 8,
            end_hour: 17,
            peak_hours: vec![9, 10, 14, 15],
            weekend_probability: 0.05,
            after_hours_probability: 0.10,
        }
    }

    /// Pattern for Asian office hours.
    pub fn asian() -> Self {
        Self {
            start_hour: 9,
            end_hour: 18,
            peak_hours: vec![10, 11, 15, 16],
            weekend_probability: 0.10,
            after_hours_probability: 0.15,
        }
    }

    /// Pattern for 24/7 batch processing.
    pub fn batch_processing() -> Self {
        Self {
            start_hour: 0,
            end_hour: 24,
            peak_hours: vec![2, 3, 4, 22, 23], // Off-peak hours for systems
            weekend_probability: 1.0,
            after_hours_probability: 1.0,
        }
    }

    /// Check if an hour is within working hours.
    pub fn is_working_hour(&self, hour: u8) -> bool {
        hour >= self.start_hour && hour < self.end_hour
    }

    /// Check if an hour is a peak hour.
    pub fn is_peak_hour(&self, hour: u8) -> bool {
        self.peak_hours.contains(&hour)
    }
}

/// Individual user account for transaction attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID (login name)
    pub user_id: String,

    /// Display name
    pub display_name: String,

    /// Email address
    pub email: Option<String>,

    /// Persona classification
    pub persona: UserPersona,

    /// Department
    pub department: Option<String>,

    /// Working hours pattern
    pub working_hours: WorkingHoursPattern,

    /// Assigned company codes
    pub company_codes: Vec<String>,

    /// Assigned cost centers (can post to)
    pub cost_centers: Vec<String>,

    /// Is this user currently active
    pub is_active: bool,

    /// Start date of employment
    pub start_date: Option<chrono::NaiveDate>,

    /// End date of employment (if terminated)
    pub end_date: Option<chrono::NaiveDate>,
}

impl User {
    /// Create a new user with minimal required fields.
    pub fn new(user_id: String, display_name: String, persona: UserPersona) -> Self {
        let working_hours = if persona.is_human() {
            WorkingHoursPattern::default()
        } else {
            WorkingHoursPattern::batch_processing()
        };

        Self {
            user_id,
            display_name,
            email: None,
            persona,
            department: None,
            working_hours,
            company_codes: Vec::new(),
            cost_centers: Vec::new(),
            is_active: true,
            start_date: None,
            end_date: None,
        }
    }

    /// Create a system/batch user.
    pub fn system(user_id: &str) -> Self {
        Self::new(
            user_id.to_string(),
            format!("System User {}", user_id),
            UserPersona::AutomatedSystem,
        )
    }

    /// Check if user can post to a company code.
    pub fn can_post_to_company(&self, company_code: &str) -> bool {
        self.company_codes.is_empty() || self.company_codes.iter().any(|c| c == company_code)
    }

    /// Generate a typical username for a persona.
    pub fn generate_username(persona: UserPersona, index: usize) -> String {
        match persona {
            UserPersona::JuniorAccountant => format!("JACC{:04}", index),
            UserPersona::SeniorAccountant => format!("SACC{:04}", index),
            UserPersona::Controller => format!("CTRL{:04}", index),
            UserPersona::Manager => format!("MGR{:04}", index),
            UserPersona::Executive => format!("EXEC{:04}", index),
            UserPersona::AutomatedSystem => format!("BATCH{:04}", index),
            UserPersona::ExternalAuditor => format!("AUDIT{:04}", index),
            UserPersona::FraudActor => format!("USER{:04}", index), // Appears normal
        }
    }
}

/// Pool of users for transaction attribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPool {
    /// All users in the pool
    pub users: Vec<User>,
    /// Index by persona for quick lookup
    #[serde(skip)]
    persona_index: std::collections::HashMap<UserPersona, Vec<usize>>,
}

impl UserPool {
    /// Create a new empty user pool.
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            persona_index: std::collections::HashMap::new(),
        }
    }

    /// Add a user to the pool.
    pub fn add_user(&mut self, user: User) {
        let idx = self.users.len();
        let persona = user.persona;
        self.users.push(user);
        self.persona_index.entry(persona).or_default().push(idx);
    }

    /// Get all users of a specific persona.
    pub fn get_users_by_persona(&self, persona: UserPersona) -> Vec<&User> {
        self.persona_index
            .get(&persona)
            .map(|indices| indices.iter().map(|&i| &self.users[i]).collect())
            .unwrap_or_default()
    }

    /// Get a random user of a specific persona.
    pub fn get_random_user(&self, persona: UserPersona, rng: &mut impl rand::Rng) -> Option<&User> {
        use rand::seq::SliceRandom;
        self.get_users_by_persona(persona).choose(rng).copied()
    }

    /// Rebuild the persona index (call after deserialization).
    pub fn rebuild_index(&mut self) {
        self.persona_index.clear();
        for (idx, user) in self.users.iter().enumerate() {
            self.persona_index
                .entry(user.persona)
                .or_default()
                .push(idx);
        }
    }

    /// Generate a standard user pool with typical distribution.
    pub fn generate_standard(company_codes: &[String]) -> Self {
        let mut pool = Self::new();

        // Junior accountants (many)
        for i in 0..10 {
            let mut user = User::new(
                User::generate_username(UserPersona::JuniorAccountant, i),
                format!("Junior Accountant {}", i + 1),
                UserPersona::JuniorAccountant,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Senior accountants
        for i in 0..5 {
            let mut user = User::new(
                User::generate_username(UserPersona::SeniorAccountant, i),
                format!("Senior Accountant {}", i + 1),
                UserPersona::SeniorAccountant,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Controllers
        for i in 0..2 {
            let mut user = User::new(
                User::generate_username(UserPersona::Controller, i),
                format!("Controller {}", i + 1),
                UserPersona::Controller,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Managers
        for i in 0..3 {
            let mut user = User::new(
                User::generate_username(UserPersona::Manager, i),
                format!("Finance Manager {}", i + 1),
                UserPersona::Manager,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        // Automated systems (many)
        for i in 0..20 {
            let mut user = User::new(
                User::generate_username(UserPersona::AutomatedSystem, i),
                format!("Batch Job {}", i + 1),
                UserPersona::AutomatedSystem,
            );
            user.company_codes = company_codes.to_vec();
            pool.add_user(user);
        }

        pool
    }
}

impl Default for UserPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_properties() {
        assert!(UserPersona::JuniorAccountant.is_human());
        assert!(!UserPersona::AutomatedSystem.is_human());
        assert!(UserPersona::Controller.has_approval_authority());
        assert!(!UserPersona::JuniorAccountant.has_approval_authority());
    }

    #[test]
    fn test_user_pool() {
        let pool = UserPool::generate_standard(&["1000".to_string()]);
        assert!(!pool.users.is_empty());
        assert!(!pool
            .get_users_by_persona(UserPersona::JuniorAccountant)
            .is_empty());
    }
}
