//! # Consumer Choice Metamodel
//!
//! This crate provides abstract traits and types for implementing the Consumer Choice Metamodel
//! as described in "Consumer Choice Metamodel: A Conceptual Validation Approach" by Amy Liffey et al.
//!
//! The Consumer Choice Metamodel is a framework for building consumer behavior simulation models.
//! This crate provides the core infrastructure and traits needed to implement agent-based models
//! of consumer decision-making processes.
//!
//! ## Features
//!
//! - **Agent-based modeling**: Define consumer agents with customizable attributes and decision-making logic
//! - **Environment modeling**: Create environments with physical and knowledge assets
//! - **Information processing**: Model information flow and transformation between agents
//! - **Flexible architecture**: Extensible factory pattern for creating model components
//! - **Validation utilities**: Built-in validation and event system for model integrity
//!
//! ## Example
//!
//! ```rust
//! use consumer_choice_metamodel::{
//!     agent::{AgentAttributes, ConsumerAgent},
//!     types::{AgentId, TriggerType},
//!     Result,
//! };
//! use std::collections::HashMap;
//!
//! // Define custom agent attributes
//! #[derive(Debug, Clone)]
//! struct MyAgentAttributes {
//!     agent_id: AgentId,
//!     income: f64,
//!     age: u32,
//! }
//!
//! impl AgentAttributes for MyAgentAttributes {
//!     fn agent_id(&self) -> &AgentId {
//!         &self.agent_id
//!     }
//!
//!     fn psychological_attributes(&self) -> HashMap<String, f64> {
//!         let mut attrs = HashMap::new();
//!         attrs.insert("risk_aversion".to_string(), 0.5);
//!         attrs.insert("environmental_concern".to_string(), 0.7);
//!         attrs.insert("price_sensitivity".to_string(), 0.8);
//!         attrs
//!     }
//!
//!     fn socioeconomic_attributes(&self) -> HashMap<String, f64> {
//!         let mut attrs = HashMap::new();
//!         attrs.insert("income".to_string(), self.income);
//!         attrs.insert("age".to_string(), self.age as f64);
//!         attrs
//!     }
//!
//!     fn stock_variables(&self) -> HashMap<String, Option<String>> {
//!         let mut vars = HashMap::new();
//!         vars.insert("car".to_string(), None);
//!         vars.insert("house".to_string(), Some("apartment".to_string()));
//!         vars
//!     }
//!
//!     fn update_attributes(&mut self, changes: HashMap<String, f64>) -> Result<()> {
//!         // Update logic here
//!         Ok(())
//!     }
//! }
//! ```

pub mod agent;
pub mod environment;
pub mod factory;
pub mod information;
pub mod model;
pub mod types;
pub mod utils;

// Re-export commonly used types and traits
pub use agent::{AgentAttributes, ChoiceModule, ConsumerAgent};
pub use environment::{
    Environment, ExogenousProcess, KnowledgeAsset, Network, PhysicalAsset, RulesOfInteraction,
};
pub use factory::ModelComponentFactory;
pub use information::{InformationDistorter, InformationFilter, Transformer};
pub use model::ConsumerChoiceModel;
pub use types::{AgentId, EvaluationDimension, TriggerType};
pub use utils::{EventBus, ModelEvent, ModelValidator};

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the Consumer Choice Metamodel
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("Model validation error: {0}")]
    Validation(String),

    #[error("Information processing error: {0}")]
    Information(String),

    #[error("Factory error: {0}")]
    Factory(String),

    #[error("Event system error: {0}")]
    Event(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_types() {
        let error = Error::Agent("test error".to_string());
        assert!(matches!(error, Error::Agent(_)));
    }
}
