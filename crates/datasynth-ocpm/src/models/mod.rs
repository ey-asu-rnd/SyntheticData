//! OCPM domain models for Object-Centric Process Mining.
//!
//! This module provides all OCPM data structures following the OCEL 2.0 standard:
//!
//! - Object types and instances with lifecycle states
//! - Activity types and event instances with lifecycle transitions
//! - Object relationships (many-to-many)
//! - Event-to-object relationships (many-to-many)
//! - Process variants and case traces
//! - Resources (users/systems performing activities)

mod activity_type;
mod event;
mod event_log;
mod object_instance;
mod object_relationship;
mod object_type;
mod process_variant;
mod resource;

pub use activity_type::*;
pub use event::*;
pub use event_log::*;
pub use object_instance::*;
pub use object_relationship::*;
pub use object_type::*;
pub use process_variant::*;
pub use resource::*;
