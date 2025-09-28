//! Validation and event system utilities for the Consumer Choice Metamodel

use crate::agent::AgentAttributes;
use crate::types::{AgentId, EvaluationDimension, SimulationTime, TriggerType};
use crate::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Event types that can occur during model execution
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EventType {
    /// Agent was added to the model
    AgentAdded,
    /// Agent was removed from the model
    AgentRemoved,
    /// Agent made a choice
    ChoiceMade,
    /// Simulation started
    SimulationStarted,
    /// Simulation paused
    SimulationPaused,
    /// Simulation resumed
    SimulationResumed,
    /// Simulation completed
    SimulationCompleted,
    /// Model validation error occurred
    ValidationError,
    /// Environment updated
    EnvironmentUpdated,
    /// Information processed
    InformationProcessed,
    /// Custom event type
    Custom(String),
}

/// Event that occurred during model execution
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ModelEvent {
    pub event_type: EventType,
    pub timestamp: SimulationTime,
    pub agent_id: Option<AgentId>,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

impl ModelEvent {
    /// Create a new model event
    pub fn new(event_type: EventType, timestamp: SimulationTime, description: String) -> Self {
        Self {
            event_type,
            timestamp,
            agent_id: None,
            description,
            metadata: HashMap::new(),
        }
    }

    /// Create an agent added event
    pub fn agent_added(agent_id: AgentId, timestamp: SimulationTime) -> Self {
        Self {
            event_type: EventType::AgentAdded,
            timestamp,
            agent_id: Some(agent_id.clone()),
            description: format!("Agent {} added to model", agent_id),
            metadata: HashMap::new(),
        }
    }

    /// Create an agent removed event
    pub fn agent_removed(agent_id: AgentId, timestamp: SimulationTime) -> Self {
        Self {
            event_type: EventType::AgentRemoved,
            timestamp,
            agent_id: Some(agent_id.clone()),
            description: format!("Agent {} removed from model", agent_id),
            metadata: HashMap::new(),
        }
    }

    /// Create a choice made event
    pub fn choice_made(
        agent_id: AgentId,
        choice_description: String,
        trigger: TriggerType,
        timestamp: SimulationTime,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("trigger".to_string(), trigger.to_string());
        metadata.insert("choice".to_string(), choice_description.clone());

        Self {
            event_type: EventType::ChoiceMade,
            timestamp,
            agent_id: Some(agent_id.clone()),
            description: format!("Agent {} made choice: {}", agent_id, choice_description),
            metadata,
        }
    }

    /// Create a simulation started event
    pub fn simulation_started(timestamp: SimulationTime) -> Self {
        Self {
            event_type: EventType::SimulationStarted,
            timestamp,
            agent_id: None,
            description: "Simulation started".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Create a simulation paused event
    pub fn simulation_paused(timestamp: SimulationTime) -> Self {
        Self {
            event_type: EventType::SimulationPaused,
            timestamp,
            agent_id: None,
            description: "Simulation paused".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Create a simulation resumed event
    pub fn simulation_resumed(timestamp: SimulationTime) -> Self {
        Self {
            event_type: EventType::SimulationResumed,
            timestamp,
            agent_id: None,
            description: "Simulation resumed".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Create a simulation completed event
    pub fn simulation_completed(timestamp: SimulationTime) -> Self {
        Self {
            event_type: EventType::SimulationCompleted,
            timestamp,
            agent_id: None,
            description: "Simulation completed".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Create a validation error event
    pub fn validation_error(error_message: String, timestamp: SimulationTime) -> Self {
        Self {
            event_type: EventType::ValidationError,
            timestamp,
            agent_id: None,
            description: format!("Validation error: {}", error_message),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add agent ID to the event
    pub fn with_agent_id(mut self, agent_id: AgentId) -> Self {
        self.agent_id = Some(agent_id);
        self
    }
}

/// Event handler trait for processing model events
pub trait EventHandler: std::fmt::Debug + Send + Sync {
    /// Handle a model event
    fn handle_event(&self, event: &ModelEvent);
}

/// Simple event handler that prints events to stdout
#[derive(Debug)]
pub struct PrintEventHandler;

impl EventHandler for PrintEventHandler {
    fn handle_event(&self, event: &ModelEvent) {
        println!(
            "[{:.2}] {:?}: {}",
            event.timestamp, event.event_type, event.description
        );
    }
}

/// Event bus for distributing events to handlers
#[derive(Debug)]
pub struct EventBus {
    handlers: Arc<Mutex<Vec<Box<dyn EventHandler>>>>,
    events: Arc<Mutex<Vec<ModelEvent>>>,
    max_events: usize,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(Mutex::new(Vec::new())),
            max_events: 10000,
        }
    }

    /// Create a new event bus with custom maximum event count
    pub fn with_max_events(max_events: usize) -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(Mutex::new(Vec::new())),
            max_events,
        }
    }

    /// Add an event handler
    pub fn add_handler(&self, handler: Box<dyn EventHandler>) {
        if let Ok(mut handlers) = self.handlers.lock() {
            handlers.push(handler);
        }
    }

    /// Emit an event to all handlers
    pub fn emit(&self, event: ModelEvent) {
        // Store the event
        if let Ok(mut events) = self.events.lock() {
            events.push(event.clone());

            // Keep only the most recent events
            if events.len() > self.max_events {
                events.remove(0);
            }
        }

        // Notify all handlers
        if let Ok(handlers) = self.handlers.lock() {
            for handler in handlers.iter() {
                handler.handle_event(&event);
            }
        }
    }

    /// Get all stored events
    pub fn get_events(&self) -> Vec<ModelEvent> {
        if let Ok(events) = self.events.lock() {
            events.clone()
        } else {
            Vec::new()
        }
    }

    /// Get events of a specific type
    pub fn get_events_of_type(&self, event_type: EventType) -> Vec<ModelEvent> {
        if let Ok(events) = self.events.lock() {
            events
                .iter()
                .filter(|event| {
                    std::mem::discriminant(&event.event_type) == std::mem::discriminant(&event_type)
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get events for a specific agent
    pub fn get_events_for_agent(&self, agent_id: &AgentId) -> Vec<ModelEvent> {
        if let Ok(events) = self.events.lock() {
            events
                .iter()
                .filter(|event| event.agent_id.as_ref() == Some(agent_id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all stored events
    pub fn clear_events(&self) {
        if let Ok(mut events) = self.events.lock() {
            events.clear();
        }
    }

    /// Get the number of stored events
    pub fn event_count(&self) -> usize {
        if let Ok(events) = self.events.lock() {
            events.len()
        } else {
            0
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation rules for model components
#[derive(Debug, Clone)]
pub struct ValidationRules {
    pub min_reliability: f64,
    pub max_reliability: f64,
    pub min_probability: f64,
    pub max_probability: f64,
    pub required_psychological_attributes: Vec<String>,
    pub required_socioeconomic_attributes: Vec<String>,
}

impl ValidationRules {
    /// Create default validation rules
    pub fn new() -> Self {
        Self {
            min_reliability: 0.0,
            max_reliability: 1.0,
            min_probability: 0.0,
            max_probability: 1.0,
            required_psychological_attributes: Vec::new(),
            required_socioeconomic_attributes: Vec::new(),
        }
    }

    /// Set required psychological attributes
    pub fn with_required_psychological_attributes(mut self, attributes: Vec<String>) -> Self {
        self.required_psychological_attributes = attributes;
        self
    }

    /// Set required socioeconomic attributes
    pub fn with_required_socioeconomic_attributes(mut self, attributes: Vec<String>) -> Self {
        self.required_socioeconomic_attributes = attributes;
        self
    }
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Validator for model components
#[derive(Debug)]
pub struct ModelValidator {
    rules: ValidationRules,
}

impl ModelValidator {
    /// Create a new model validator with default rules
    pub fn new() -> Self {
        Self {
            rules: ValidationRules::new(),
        }
    }

    /// Create a new model validator with custom rules
    pub fn with_rules(rules: ValidationRules) -> Self {
        Self { rules }
    }

    /// Validate agent attributes
    pub fn validate_agent_attributes(&self, attributes: &dyn AgentAttributes) -> Result<()> {
        let psychological = attributes.psychological_attributes();
        let socioeconomic = attributes.socioeconomic_attributes();

        // Check required psychological attributes
        for required_attr in &self.rules.required_psychological_attributes {
            if !psychological.contains_key(required_attr) {
                return Err(Error::Validation(format!(
                    "Missing required psychological attribute: {}",
                    required_attr
                )));
            }
        }

        // Check required socioeconomic attributes
        for required_attr in &self.rules.required_socioeconomic_attributes {
            if !socioeconomic.contains_key(required_attr) {
                return Err(Error::Validation(format!(
                    "Missing required socioeconomic attribute: {}",
                    required_attr
                )));
            }
        }

        // Validate psychological attribute values
        for (name, value) in psychological {
            if value < 0.0 || value > 1.0 {
                return Err(Error::Validation(format!(
                    "Psychological attribute '{}' must be between 0.0 and 1.0, got {}",
                    name, value
                )));
            }
        }

        // Validate socioeconomic attribute values (allow any positive values)
        for (name, value) in socioeconomic {
            if value < 0.0 {
                return Err(Error::Validation(format!(
                    "Socioeconomic attribute '{}' must be non-negative, got {}",
                    name, value
                )));
            }
        }

        Ok(())
    }

    /// Validate probability value
    pub fn validate_probability(&self, value: f64, name: &str) -> Result<()> {
        if value < self.rules.min_probability || value > self.rules.max_probability {
            return Err(Error::Validation(format!(
                "Probability '{}' must be between {} and {}, got {}",
                name, self.rules.min_probability, self.rules.max_probability, value
            )));
        }
        Ok(())
    }

    /// Validate reliability value
    pub fn validate_reliability(&self, value: f64, name: &str) -> Result<()> {
        if value < self.rules.min_reliability || value > self.rules.max_reliability {
            return Err(Error::Validation(format!(
                "Reliability '{}' must be between {} and {}, got {}",
                name, self.rules.min_reliability, self.rules.max_reliability, value
            )));
        }
        Ok(())
    }

    /// Validate evaluation scores
    pub fn validate_evaluation_scores(
        &self,
        scores: &HashMap<EvaluationDimension, f64>,
    ) -> Result<()> {
        for (dimension, score) in scores {
            self.validate_probability(*score, &format!("evaluation score for {:?}", dimension))?;
        }
        Ok(())
    }

    /// Get the validation rules
    pub fn rules(&self) -> &ValidationRules {
        &self.rules
    }

    /// Update validation rules
    pub fn set_rules(&mut self, rules: ValidationRules) {
        self.rules = rules;
    }
}

impl Default for ModelValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::BasicAgentAttributes;

    #[test]
    fn test_model_event_creation() {
        let agent_id = AgentId::new();
        let event = ModelEvent::agent_added(agent_id.clone(), 10.0);

        assert!(matches!(event.event_type, EventType::AgentAdded));
        assert_eq!(event.timestamp, 10.0);
        assert_eq!(event.agent_id, Some(agent_id));
    }

    #[test]
    fn test_event_bus() {
        let event_bus = EventBus::new();
        let agent_id = AgentId::new();
        let event = ModelEvent::agent_added(agent_id.clone(), 5.0);

        event_bus.emit(event);

        assert_eq!(event_bus.event_count(), 1);

        let events = event_bus.get_events_for_agent(&agent_id);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_model_validator() {
        let validator = ModelValidator::new();
        let agent_id = AgentId::new();

        let valid_attributes = BasicAgentAttributes::new(agent_id)
            .with_psychological_attribute("risk_aversion".to_string(), 0.5)
            .with_socioeconomic_attribute("income".to_string(), 50000.0);

        assert!(validator
            .validate_agent_attributes(&valid_attributes)
            .is_ok());
    }

    #[test]
    fn test_validator_probability() {
        let validator = ModelValidator::new();

        assert!(validator.validate_probability(0.5, "test").is_ok());
        assert!(validator.validate_probability(-0.1, "test").is_err());
        assert!(validator.validate_probability(1.1, "test").is_err());
    }

    #[test]
    fn test_validation_rules() {
        let rules = ValidationRules::new()
            .with_required_psychological_attributes(vec!["risk_aversion".to_string()])
            .with_required_socioeconomic_attributes(vec!["income".to_string()]);

        let validator = ModelValidator::with_rules(rules);
        let agent_id = AgentId::new();

        // Missing required attributes
        let invalid_attributes = BasicAgentAttributes::new(agent_id.clone());
        assert!(validator
            .validate_agent_attributes(&invalid_attributes)
            .is_err());

        // Valid attributes
        let valid_attributes = BasicAgentAttributes::new(agent_id)
            .with_psychological_attribute("risk_aversion".to_string(), 0.5)
            .with_socioeconomic_attribute("income".to_string(), 50000.0);
        assert!(validator
            .validate_agent_attributes(&valid_attributes)
            .is_ok());
    }
}
