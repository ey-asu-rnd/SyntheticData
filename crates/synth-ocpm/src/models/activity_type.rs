//! Activity type definitions for OCPM.
//!
//! Activity types define the schema for business activities that can occur
//! on objects, including which object types they affect and what state
//! transitions they trigger.

use serde::{Deserialize, Serialize};
use synth_core::models::BusinessProcess;

/// Definition of a business activity in OCPM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityType {
    /// Unique activity type ID
    pub activity_id: String,
    /// Human-readable name (e.g., "Create Order", "Post Invoice")
    pub name: String,
    /// Business process this activity belongs to
    pub business_process: BusinessProcess,
    /// Object types this activity operates on
    pub involved_object_types: Vec<String>,
    /// Lifecycle transitions this activity triggers
    pub state_transitions: Vec<ActivityStateTransition>,
    /// Resource types that can perform this activity
    pub allowed_resource_types: Vec<String>,
    /// Typical duration in minutes (for simulation)
    pub typical_duration_minutes: Option<f64>,
    /// Standard deviation of duration
    pub duration_std_dev: Option<f64>,
    /// Is this a manual or automated activity
    pub is_automated: bool,
    /// Does this activity create a new object
    pub creates_object: bool,
    /// Does this activity complete/terminate an object
    pub completes_object: bool,
}

impl ActivityType {
    /// Create a new activity type.
    pub fn new(activity_id: &str, name: &str, business_process: BusinessProcess) -> Self {
        Self {
            activity_id: activity_id.into(),
            name: name.into(),
            business_process,
            involved_object_types: Vec::new(),
            state_transitions: Vec::new(),
            allowed_resource_types: vec!["user".into()],
            typical_duration_minutes: None,
            duration_std_dev: None,
            is_automated: false,
            creates_object: false,
            completes_object: false,
        }
    }

    /// Add involved object types.
    pub fn with_object_types(mut self, types: Vec<&str>) -> Self {
        self.involved_object_types = types.into_iter().map(String::from).collect();
        self
    }

    /// Add state transitions.
    pub fn with_transitions(mut self, transitions: Vec<ActivityStateTransition>) -> Self {
        self.state_transitions = transitions;
        self
    }

    /// Set typical duration.
    pub fn with_duration(mut self, minutes: f64, std_dev: f64) -> Self {
        self.typical_duration_minutes = Some(minutes);
        self.duration_std_dev = Some(std_dev);
        self
    }

    /// Mark as automated.
    pub fn automated(mut self) -> Self {
        self.is_automated = true;
        self
    }

    /// Mark as creating an object.
    pub fn creates(mut self) -> Self {
        self.creates_object = true;
        self
    }

    /// Mark as completing an object.
    pub fn completes(mut self) -> Self {
        self.completes_object = true;
        self
    }

    // ========== P2P Activities ==========

    /// Create Purchase Order activity.
    pub fn create_po() -> Self {
        Self::new("create_po", "Create Purchase Order", BusinessProcess::P2P)
            .with_object_types(vec!["purchase_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "purchase_order",
                None,
                "created",
            )])
            .with_duration(15.0, 5.0)
            .creates()
    }

    /// Approve Purchase Order activity.
    pub fn approve_po() -> Self {
        Self::new("approve_po", "Approve Purchase Order", BusinessProcess::P2P)
            .with_object_types(vec!["purchase_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "purchase_order",
                Some("created"),
                "approved",
            )])
            .with_duration(30.0, 15.0)
    }

    /// Release Purchase Order activity.
    pub fn release_po() -> Self {
        Self::new("release_po", "Release Purchase Order", BusinessProcess::P2P)
            .with_object_types(vec!["purchase_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "purchase_order",
                Some("approved"),
                "released",
            )])
            .with_duration(5.0, 2.0)
            .automated()
    }

    /// Create Goods Receipt activity.
    pub fn create_gr() -> Self {
        Self::new("create_gr", "Create Goods Receipt", BusinessProcess::P2P)
            .with_object_types(vec!["goods_receipt", "purchase_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("goods_receipt", None, "created"),
                ActivityStateTransition::new("purchase_order", Some("released"), "received"),
            ])
            .with_duration(10.0, 5.0)
            .creates()
    }

    /// Post Goods Receipt activity.
    pub fn post_gr() -> Self {
        Self::new("post_gr", "Post Goods Receipt", BusinessProcess::P2P)
            .with_object_types(vec!["goods_receipt"])
            .with_transitions(vec![ActivityStateTransition::new(
                "goods_receipt",
                Some("created"),
                "posted",
            )])
            .with_duration(2.0, 1.0)
            .automated()
    }

    /// Receive Invoice activity.
    pub fn receive_invoice() -> Self {
        Self::new("receive_invoice", "Receive Invoice", BusinessProcess::P2P)
            .with_object_types(vec!["vendor_invoice"])
            .with_transitions(vec![ActivityStateTransition::new(
                "vendor_invoice",
                None,
                "received",
            )])
            .with_duration(5.0, 2.0)
            .creates()
    }

    /// Verify Invoice (three-way match) activity.
    pub fn verify_invoice() -> Self {
        Self::new("verify_invoice", "Verify Invoice", BusinessProcess::P2P)
            .with_object_types(vec!["vendor_invoice", "purchase_order", "goods_receipt"])
            .with_transitions(vec![ActivityStateTransition::new(
                "vendor_invoice",
                Some("received"),
                "verified",
            )])
            .with_duration(20.0, 10.0)
    }

    /// Post Invoice activity.
    pub fn post_invoice() -> Self {
        Self::new("post_invoice", "Post Invoice", BusinessProcess::P2P)
            .with_object_types(vec!["vendor_invoice", "purchase_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("vendor_invoice", Some("verified"), "posted"),
                ActivityStateTransition::new("purchase_order", Some("received"), "invoiced"),
            ])
            .with_duration(3.0, 1.0)
            .automated()
    }

    /// Execute Payment activity.
    pub fn execute_payment() -> Self {
        Self::new("execute_payment", "Execute Payment", BusinessProcess::P2P)
            .with_object_types(vec!["vendor_invoice", "purchase_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("vendor_invoice", Some("posted"), "paid"),
                ActivityStateTransition::new("purchase_order", Some("invoiced"), "paid"),
            ])
            .with_duration(1.0, 0.5)
            .automated()
            .completes()
    }

    // ========== O2C Activities ==========

    /// Create Sales Order activity.
    pub fn create_so() -> Self {
        Self::new("create_so", "Create Sales Order", BusinessProcess::O2C)
            .with_object_types(vec!["sales_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "sales_order",
                None,
                "created",
            )])
            .with_duration(10.0, 5.0)
            .creates()
    }

    /// Check Credit activity.
    pub fn check_credit() -> Self {
        Self::new("check_credit", "Check Credit", BusinessProcess::O2C)
            .with_object_types(vec!["sales_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "sales_order",
                Some("created"),
                "credit_checked",
            )])
            .with_duration(2.0, 1.0)
            .automated()
    }

    /// Release Sales Order activity.
    pub fn release_so() -> Self {
        Self::new("release_so", "Release Sales Order", BusinessProcess::O2C)
            .with_object_types(vec!["sales_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "sales_order",
                Some("credit_checked"),
                "released",
            )])
            .with_duration(5.0, 2.0)
    }

    /// Create Delivery activity.
    pub fn create_delivery() -> Self {
        Self::new("create_delivery", "Create Delivery", BusinessProcess::O2C)
            .with_object_types(vec!["delivery", "sales_order"])
            .with_transitions(vec![ActivityStateTransition::new(
                "delivery", None, "created",
            )])
            .with_duration(5.0, 2.0)
            .creates()
    }

    /// Pick activity.
    pub fn pick() -> Self {
        Self::new("pick", "Pick", BusinessProcess::O2C)
            .with_object_types(vec!["delivery"])
            .with_transitions(vec![ActivityStateTransition::new(
                "delivery",
                Some("created"),
                "picked",
            )])
            .with_duration(30.0, 15.0)
    }

    /// Pack activity.
    pub fn pack() -> Self {
        Self::new("pack", "Pack", BusinessProcess::O2C)
            .with_object_types(vec!["delivery"])
            .with_transitions(vec![ActivityStateTransition::new(
                "delivery",
                Some("picked"),
                "packed",
            )])
            .with_duration(20.0, 10.0)
    }

    /// Ship activity.
    pub fn ship() -> Self {
        Self::new("ship", "Ship", BusinessProcess::O2C)
            .with_object_types(vec!["delivery", "sales_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("delivery", Some("packed"), "shipped"),
                ActivityStateTransition::new("sales_order", Some("released"), "delivered"),
            ])
            .with_duration(10.0, 5.0)
    }

    /// Create Customer Invoice activity.
    pub fn create_customer_invoice() -> Self {
        Self::new(
            "create_customer_invoice",
            "Create Customer Invoice",
            BusinessProcess::O2C,
        )
        .with_object_types(vec!["customer_invoice", "sales_order"])
        .with_transitions(vec![
            ActivityStateTransition::new("customer_invoice", None, "created"),
            ActivityStateTransition::new("sales_order", Some("delivered"), "invoiced"),
        ])
        .with_duration(5.0, 2.0)
        .creates()
    }

    /// Post Customer Invoice activity.
    pub fn post_customer_invoice() -> Self {
        Self::new(
            "post_customer_invoice",
            "Post Customer Invoice",
            BusinessProcess::O2C,
        )
        .with_object_types(vec!["customer_invoice"])
        .with_transitions(vec![ActivityStateTransition::new(
            "customer_invoice",
            Some("created"),
            "posted",
        )])
        .with_duration(2.0, 1.0)
        .automated()
    }

    /// Receive Payment activity.
    pub fn receive_payment() -> Self {
        Self::new("receive_payment", "Receive Payment", BusinessProcess::O2C)
            .with_object_types(vec!["customer_invoice", "sales_order"])
            .with_transitions(vec![
                ActivityStateTransition::new("customer_invoice", Some("posted"), "paid"),
                ActivityStateTransition::new("sales_order", Some("invoiced"), "paid"),
            ])
            .with_duration(1.0, 0.5)
            .automated()
            .completes()
    }

    /// Get all standard P2P activities.
    pub fn p2p_activities() -> Vec<Self> {
        vec![
            Self::create_po(),
            Self::approve_po(),
            Self::release_po(),
            Self::create_gr(),
            Self::post_gr(),
            Self::receive_invoice(),
            Self::verify_invoice(),
            Self::post_invoice(),
            Self::execute_payment(),
        ]
    }

    /// Get all standard O2C activities.
    pub fn o2c_activities() -> Vec<Self> {
        vec![
            Self::create_so(),
            Self::check_credit(),
            Self::release_so(),
            Self::create_delivery(),
            Self::pick(),
            Self::pack(),
            Self::ship(),
            Self::create_customer_invoice(),
            Self::post_customer_invoice(),
            Self::receive_payment(),
        ]
    }
}

/// State transition triggered by an activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityStateTransition {
    /// Object type ID affected
    pub object_type_id: String,
    /// From state (None = any state, including initial)
    pub from_state: Option<String>,
    /// To state
    pub to_state: String,
}

impl ActivityStateTransition {
    /// Create a new state transition.
    pub fn new(object_type_id: &str, from_state: Option<&str>, to_state: &str) -> Self {
        Self {
            object_type_id: object_type_id.into(),
            from_state: from_state.map(String::from),
            to_state: to_state.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_type_creation() {
        let activity = ActivityType::create_po();
        assert_eq!(activity.activity_id, "create_po");
        assert!(activity.creates_object);
        assert!(!activity.is_automated);
    }

    #[test]
    fn test_p2p_activities() {
        let activities = ActivityType::p2p_activities();
        assert_eq!(activities.len(), 9);
    }

    #[test]
    fn test_o2c_activities() {
        let activities = ActivityType::o2c_activities();
        assert_eq!(activities.len(), 10);
    }
}
