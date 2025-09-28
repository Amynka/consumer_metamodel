//! Core types and enumerations for the Consumer Choice Metamodel

use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Unique identifier for agents in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AgentId(Uuid);

impl AgentId {
    /// Create a new random AgentId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an AgentId from a string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Enumeration of decision triggers that can cause agents to make choices
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TriggerType {
    /// Decision triggered by time passage
    Temporal,
    /// Decision triggered by information received
    Informational,
    /// Decision triggered by social influence
    Social,
    /// Decision triggered by economic factors
    Economic,
    /// Decision triggered by regulatory changes
    Regulatory,
    /// Decision triggered by technological changes
    Technological,
    /// Decision triggered by environmental factors
    Environmental,
    /// Decision triggered by personal circumstances
    Personal,
    /// Decision triggered by random events
    Stochastic,
    /// Custom trigger type
    Custom(String),
}

impl TriggerType {
    /// Get all standard trigger types (excluding Custom)
    pub fn standard_types() -> Vec<TriggerType> {
        vec![
            TriggerType::Temporal,
            TriggerType::Informational,
            TriggerType::Social,
            TriggerType::Economic,
            TriggerType::Regulatory,
            TriggerType::Technological,
            TriggerType::Environmental,
            TriggerType::Personal,
            TriggerType::Stochastic,
        ]
    }
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerType::Temporal => write!(f, "Temporal"),
            TriggerType::Informational => write!(f, "Informational"),
            TriggerType::Social => write!(f, "Social"),
            TriggerType::Economic => write!(f, "Economic"),
            TriggerType::Regulatory => write!(f, "Regulatory"),
            TriggerType::Technological => write!(f, "Technological"),
            TriggerType::Environmental => write!(f, "Environmental"),
            TriggerType::Personal => write!(f, "Personal"),
            TriggerType::Stochastic => write!(f, "Stochastic"),
            TriggerType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Enumeration of choice evaluation dimensions used by agents to assess options
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EvaluationDimension {
    /// Economic/financial considerations
    Economic,
    /// Environmental impact considerations
    Environmental,
    /// Social factors and peer influence
    Social,
    /// Functional performance and utility
    Functional,
    /// Aesthetic and design considerations
    Aesthetic,
    /// Convenience and ease of use
    Convenience,
    /// Safety and security considerations
    Safety,
    /// Reliability and durability
    Reliability,
    /// Innovation and technology
    Innovation,
    /// Brand reputation and trust
    Brand,
    /// Custom evaluation dimension
    Custom(String),
}

impl EvaluationDimension {
    /// Get all standard evaluation dimensions (excluding Custom)
    pub fn standard_dimensions() -> Vec<EvaluationDimension> {
        vec![
            EvaluationDimension::Economic,
            EvaluationDimension::Environmental,
            EvaluationDimension::Social,
            EvaluationDimension::Functional,
            EvaluationDimension::Aesthetic,
            EvaluationDimension::Convenience,
            EvaluationDimension::Safety,
            EvaluationDimension::Reliability,
            EvaluationDimension::Innovation,
            EvaluationDimension::Brand,
        ]
    }
}

impl std::fmt::Display for EvaluationDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationDimension::Economic => write!(f, "Economic"),
            EvaluationDimension::Environmental => write!(f, "Environmental"),
            EvaluationDimension::Social => write!(f, "Social"),
            EvaluationDimension::Functional => write!(f, "Functional"),
            EvaluationDimension::Aesthetic => write!(f, "Aesthetic"),
            EvaluationDimension::Convenience => write!(f, "Convenience"),
            EvaluationDimension::Safety => write!(f, "Safety"),
            EvaluationDimension::Reliability => write!(f, "Reliability"),
            EvaluationDimension::Innovation => write!(f, "Innovation"),
            EvaluationDimension::Brand => write!(f, "Brand"),
            EvaluationDimension::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Unique identifier for assets in the environment
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AssetId(Uuid);

impl AssetId {
    /// Create a new random AssetId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an AssetId from a string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for AssetId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for model instances
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModelId(Uuid);

impl ModelId {
    /// Create a new random ModelId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a ModelId from a string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ModelId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Time representation for the simulation
pub type SimulationTime = f64;

/// Monetary value representation
pub type MonetaryValue = f64;

/// Probability value (0.0 to 1.0)
pub type Probability = f64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_creation() {
        let id1 = AgentId::new();
        let id2 = AgentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_agent_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = AgentId::from_string(uuid_str).unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn test_trigger_type_display() {
        assert_eq!(TriggerType::Economic.to_string(), "Economic");
        assert_eq!(
            TriggerType::Custom("test".to_string()).to_string(),
            "Custom(test)"
        );
    }

    #[test]
    fn test_evaluation_dimension_standard_dimensions() {
        let dimensions = EvaluationDimension::standard_dimensions();
        assert!(dimensions.contains(&EvaluationDimension::Economic));
        assert!(dimensions.contains(&EvaluationDimension::Environmental));
        assert_eq!(dimensions.len(), 10);
    }

    #[test]
    fn test_trigger_type_standard_types() {
        let types = TriggerType::standard_types();
        assert!(types.contains(&TriggerType::Temporal));
        assert!(types.contains(&TriggerType::Stochastic));
        assert_eq!(types.len(), 9);
    }
}
