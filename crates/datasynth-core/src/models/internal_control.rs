//! Internal Controls System (ICS) definitions for SOX compliance.
//!
//! Provides structures for modeling internal controls, control testing,
//! and SOX 404 compliance markers in synthetic accounting data.

use serde::{Deserialize, Serialize};

use super::user::UserPersona;

/// Control type based on SOX 404 framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlType {
    /// Prevents errors/fraud before they occur
    Preventive,
    /// Detects errors/fraud after they occur
    Detective,
    /// Continuous monitoring and analytics
    Monitoring,
}

impl std::fmt::Display for ControlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preventive => write!(f, "Preventive"),
            Self::Detective => write!(f, "Detective"),
            Self::Monitoring => write!(f, "Monitoring"),
        }
    }
}

/// Control testing frequency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlFrequency {
    /// Applied to every transaction
    Transactional,
    /// Performed daily
    Daily,
    /// Performed weekly
    Weekly,
    /// Performed monthly
    Monthly,
    /// Performed quarterly
    Quarterly,
    /// Performed annually
    Annual,
}

impl std::fmt::Display for ControlFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transactional => write!(f, "Transactional"),
            Self::Daily => write!(f, "Daily"),
            Self::Weekly => write!(f, "Weekly"),
            Self::Monthly => write!(f, "Monthly"),
            Self::Quarterly => write!(f, "Quarterly"),
            Self::Annual => write!(f, "Annual"),
        }
    }
}

/// Risk level for controls and control deficiencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Low risk - minor impact
    Low,
    /// Medium risk - moderate impact
    Medium,
    /// High risk - significant impact
    High,
    /// Critical risk - material impact on financial statements
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// SOX 404 financial statement assertions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoxAssertion {
    /// Transactions and events have been recorded
    Existence,
    /// All transactions have been recorded
    Completeness,
    /// Amounts are recorded at appropriate values
    Valuation,
    /// Entity has rights to assets and obligations for liabilities
    RightsAndObligations,
    /// Components are properly classified and disclosed
    PresentationAndDisclosure,
}

impl std::fmt::Display for SoxAssertion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Existence => write!(f, "Existence"),
            Self::Completeness => write!(f, "Completeness"),
            Self::Valuation => write!(f, "Valuation"),
            Self::RightsAndObligations => write!(f, "RightsAndObligations"),
            Self::PresentationAndDisclosure => write!(f, "PresentationAndDisclosure"),
        }
    }
}

/// Control status for transaction-level tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    /// Control operating effectively
    #[default]
    Effective,
    /// Control exception/deficiency found
    Exception,
    /// Control not yet tested
    NotTested,
    /// Exception has been remediated
    Remediated,
}

impl std::fmt::Display for ControlStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Effective => write!(f, "Effective"),
            Self::Exception => write!(f, "Exception"),
            Self::NotTested => write!(f, "NotTested"),
            Self::Remediated => write!(f, "Remediated"),
        }
    }
}

/// Internal control definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalControl {
    /// Unique control identifier (e.g., "C001", "C010")
    pub control_id: String,
    /// Control name/title
    pub control_name: String,
    /// Type of control (Preventive, Detective, Monitoring)
    pub control_type: ControlType,
    /// Control objective description
    pub objective: String,
    /// How often the control is performed
    pub frequency: ControlFrequency,
    /// Role responsible for executing/owning the control
    pub owner_role: UserPersona,
    /// Risk level associated with control failure
    pub risk_level: RiskLevel,
    /// Detailed description of the control procedure
    pub description: String,
    /// Whether this is a SOX 404 key control
    pub is_key_control: bool,
    /// SOX assertion this control addresses
    pub sox_assertion: SoxAssertion,
}

impl InternalControl {
    /// Create a new internal control.
    pub fn new(
        control_id: impl Into<String>,
        control_name: impl Into<String>,
        control_type: ControlType,
        objective: impl Into<String>,
    ) -> Self {
        Self {
            control_id: control_id.into(),
            control_name: control_name.into(),
            control_type,
            objective: objective.into(),
            frequency: ControlFrequency::Transactional,
            owner_role: UserPersona::Controller,
            risk_level: RiskLevel::Medium,
            description: String::new(),
            is_key_control: false,
            sox_assertion: SoxAssertion::Existence,
        }
    }

    /// Builder method to set frequency.
    pub fn with_frequency(mut self, frequency: ControlFrequency) -> Self {
        self.frequency = frequency;
        self
    }

    /// Builder method to set owner role.
    pub fn with_owner(mut self, owner: UserPersona) -> Self {
        self.owner_role = owner;
        self
    }

    /// Builder method to set risk level.
    pub fn with_risk_level(mut self, level: RiskLevel) -> Self {
        self.risk_level = level;
        self
    }

    /// Builder method to set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder method to mark as key control.
    pub fn as_key_control(mut self) -> Self {
        self.is_key_control = true;
        self
    }

    /// Builder method to set SOX assertion.
    pub fn with_assertion(mut self, assertion: SoxAssertion) -> Self {
        self.sox_assertion = assertion;
        self
    }

    /// Generate standard controls for a typical organization.
    pub fn standard_controls() -> Vec<Self> {
        vec![
            // Cash controls
            Self::new(
                "C001",
                "Cash Account Daily Review",
                ControlType::Detective,
                "Review all cash transactions daily for unauthorized activity",
            )
            .with_frequency(ControlFrequency::Daily)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "Daily reconciliation of cash accounts with bank statements and review of unusual transactions",
            ),

            // Large transaction approval
            Self::new(
                "C002",
                "Large Transaction Multi-Level Approval",
                ControlType::Preventive,
                "Transactions over $10,000 require additional approval levels",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::Manager)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "Multi-level approval workflow for transactions exceeding defined thresholds",
            ),

            // P2P Three-Way Match
            Self::new(
                "C010",
                "Three-Way Match",
                ControlType::Preventive,
                "Match purchase order, receipt, and invoice before payment",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::SeniorAccountant)
            .with_risk_level(RiskLevel::Medium)
            .as_key_control()
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Automated matching of PO, goods receipt, and vendor invoice prior to payment release",
            ),

            // Vendor Master Maintenance
            Self::new(
                "C011",
                "Vendor Master Data Maintenance",
                ControlType::Preventive,
                "Segregated access for vendor master data changes",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::SeniorAccountant)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "Restricted access to vendor master data with dual-approval for bank account changes",
            ),

            // O2C Revenue Recognition
            Self::new(
                "C020",
                "Revenue Recognition Review",
                ControlType::Detective,
                "Review revenue entries for proper timing and classification",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::Critical)
            .as_key_control()
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "Monthly review of revenue recognition to ensure compliance with ASC 606",
            ),

            // Credit Limit Enforcement
            Self::new(
                "C021",
                "Customer Credit Limit Check",
                ControlType::Preventive,
                "Automatic credit limit check before order acceptance",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::AutomatedSystem)
            .with_risk_level(RiskLevel::Medium)
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "System-enforced credit limit validation at order entry",
            ),

            // GL Account Reconciliation
            Self::new(
                "C030",
                "GL Account Reconciliation",
                ControlType::Detective,
                "Monthly reconciliation of all balance sheet accounts",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::SeniorAccountant)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Complete reconciliation of all balance sheet accounts with supporting documentation",
            ),

            // Journal Entry Review
            Self::new(
                "C031",
                "Manual Journal Entry Review",
                ControlType::Detective,
                "Review of all manual journal entries over threshold",
            )
            .with_frequency(ControlFrequency::Daily)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "Daily review of manual journal entries with supporting documentation",
            ),

            // Period Close Review
            Self::new(
                "C032",
                "Period Close Checklist",
                ControlType::Detective,
                "Comprehensive checklist for period-end close procedures",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::Medium)
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Standardized period-end close checklist ensuring all procedures completed",
            ),

            // Payroll Processing
            Self::new(
                "C040",
                "Payroll Processing Review",
                ControlType::Detective,
                "Review of payroll processing for accuracy",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::Controller)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Valuation)
            .with_description(
                "Monthly review of payroll journals and reconciliation to HR records",
            ),

            // Fixed Asset Additions
            Self::new(
                "C050",
                "Fixed Asset Addition Approval",
                ControlType::Preventive,
                "Multi-level approval for capital expenditures",
            )
            .with_frequency(ControlFrequency::Transactional)
            .with_owner(UserPersona::Manager)
            .with_risk_level(RiskLevel::Medium)
            .with_assertion(SoxAssertion::Existence)
            .with_description(
                "Approval workflow for capital asset additions based on dollar thresholds",
            ),

            // Intercompany Reconciliation
            Self::new(
                "C060",
                "Intercompany Balance Reconciliation",
                ControlType::Detective,
                "Monthly reconciliation of intercompany balances",
            )
            .with_frequency(ControlFrequency::Monthly)
            .with_owner(UserPersona::SeniorAccountant)
            .with_risk_level(RiskLevel::High)
            .as_key_control()
            .with_assertion(SoxAssertion::Completeness)
            .with_description(
                "Full reconciliation of intercompany accounts between all entities",
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_creation() {
        let control = InternalControl::new(
            "TEST001",
            "Test Control",
            ControlType::Preventive,
            "Test objective",
        )
        .with_frequency(ControlFrequency::Daily)
        .with_risk_level(RiskLevel::High)
        .as_key_control();

        assert_eq!(control.control_id, "TEST001");
        assert_eq!(control.control_type, ControlType::Preventive);
        assert_eq!(control.frequency, ControlFrequency::Daily);
        assert_eq!(control.risk_level, RiskLevel::High);
        assert!(control.is_key_control);
    }

    #[test]
    fn test_standard_controls() {
        let controls = InternalControl::standard_controls();
        assert!(!controls.is_empty());

        // Verify key controls exist
        let key_controls: Vec<_> = controls.iter().filter(|c| c.is_key_control).collect();
        assert!(key_controls.len() >= 5);

        // Verify different control types exist
        let preventive: Vec<_> = controls
            .iter()
            .filter(|c| c.control_type == ControlType::Preventive)
            .collect();
        let detective: Vec<_> = controls
            .iter()
            .filter(|c| c.control_type == ControlType::Detective)
            .collect();

        assert!(!preventive.is_empty());
        assert!(!detective.is_empty());
    }

    #[test]
    fn test_control_status_display() {
        assert_eq!(ControlStatus::Effective.to_string(), "Effective");
        assert_eq!(ControlStatus::Exception.to_string(), "Exception");
    }
}
