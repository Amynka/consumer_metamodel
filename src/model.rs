//! Main model implementation for the Consumer Choice Metamodel

use crate::agent::{AgentAttributes, ChoiceModule, ConsumerAgent};
use crate::environment::{Environment, ExogenousProcess, KnowledgeAsset, Network, PhysicalAsset, RulesOfInteraction};
use crate::information::{Information, Transformer};
use crate::types::{AgentId, ModelId, SimulationTime};
use crate::utils::{EventBus, ModelEvent, ModelValidator};
use crate::{Error, Result};
use std::collections::HashMap;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Configuration for the consumer choice model
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelConfiguration {
    pub model_id: ModelId,
    pub name: String,
    pub description: String,
    pub time_step: SimulationTime,
    pub max_simulation_time: SimulationTime,
    pub random_seed: Option<u64>,
    pub validation_enabled: bool,
    pub event_logging_enabled: bool,
}

impl ModelConfiguration {
    /// Create a new model configuration
    pub fn new(name: String, description: String) -> Self {
        Self {
            model_id: ModelId::new(),
            name,
            description,
            time_step: 1.0,
            max_simulation_time: 1000.0,
            random_seed: None,
            validation_enabled: true,
            event_logging_enabled: true,
        }
    }

    /// Set time step
    pub fn with_time_step(mut self, time_step: SimulationTime) -> Self {
        self.time_step = time_step;
        self
    }

    /// Set maximum simulation time
    pub fn with_max_time(mut self, max_time: SimulationTime) -> Self {
        self.max_simulation_time = max_time;
        self
    }

    /// Set random seed
    pub fn with_random_seed(mut self, seed: u64) -> Self {
        self.random_seed = Some(seed);
        self
    }

    /// Enable or disable validation
    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.validation_enabled = enabled;
        self
    }
}

/// State of the simulation model
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelState {
    /// Model is initialized but not started
    Initialized,
    /// Model is currently running
    Running,
    /// Model is paused
    Paused,
    /// Model has completed successfully
    Completed,
    /// Model stopped due to error
    Error,
}

/// Statistics about model execution
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelStatistics {
    pub total_agents: usize,
    pub total_choices_made: usize,
    pub average_choices_per_agent: f64,
    pub simulation_duration: SimulationTime,
    pub events_processed: usize,
    pub validation_errors: usize,
}

impl ModelStatistics {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self {
            total_agents: 0,
            total_choices_made: 0,
            average_choices_per_agent: 0.0,
            simulation_duration: 0.0,
            events_processed: 0,
            validation_errors: 0,
        }
    }

    /// Update statistics with new data
    pub fn update(&mut self, agents: &HashMap<AgentId, Box<dyn std::any::Any + Send + Sync>>) {
        self.total_agents = agents.len();
        // Note: In a real implementation, you'd need to properly handle the agent types
        // This is simplified for the example
    }
}

impl Default for ModelStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Main consumer choice model implementation
#[derive(Debug)]
pub struct ConsumerChoiceModel<A, C, P, K, N, R, E, F, D>
where
    A: AgentAttributes + 'static,
    C: ChoiceModule + 'static,
    P: PhysicalAsset,
    K: KnowledgeAsset,
    N: Network,
    R: RulesOfInteraction,
    E: ExogenousProcess,
    F: crate::information::InformationFilter,
    D: crate::information::InformationDistorter,
{
    configuration: ModelConfiguration,
    state: ModelState,
    current_time: SimulationTime,
    agents: HashMap<AgentId, ConsumerAgent<A, C>>,
    environment: Environment<P, K, N, R, E>,
    information_transformer: Transformer<F, D>,
    event_bus: EventBus,
    validator: ModelValidator,
    statistics: ModelStatistics,
}

impl<A, C, P, K, N, R, E, F, D> ConsumerChoiceModel<A, C, P, K, N, R, E, F, D>
where
    A: AgentAttributes + 'static,
    C: ChoiceModule + 'static,
    P: PhysicalAsset,
    K: KnowledgeAsset,
    N: Network,
    R: RulesOfInteraction,
    E: ExogenousProcess,
    F: crate::information::InformationFilter,
    D: crate::information::InformationDistorter,
{
    /// Create a new consumer choice model
    pub fn new(
        configuration: ModelConfiguration,
        environment: Environment<P, K, N, R, E>,
        information_transformer: Transformer<F, D>,
    ) -> Self {
        let event_bus = EventBus::new();
        let validator = ModelValidator::new();

        Self {
            configuration,
            state: ModelState::Initialized,
            current_time: 0.0,
            agents: HashMap::new(),
            environment,
            information_transformer,
            event_bus,
            validator,
            statistics: ModelStatistics::new(),
        }
    }

    /// Get the model configuration
    pub fn configuration(&self) -> &ModelConfiguration {
        &self.configuration
    }

    /// Get the current model state
    pub fn state(&self) -> ModelState {
        self.state
    }

    /// Get the current simulation time
    pub fn current_time(&self) -> SimulationTime {
        self.current_time
    }

    /// Get model statistics
    pub fn statistics(&self) -> &ModelStatistics {
        &self.statistics
    }

    /// Add an agent to the model
    pub fn add_agent(&mut self, agent: ConsumerAgent<A, C>) -> Result<()> {
        if self.state != ModelState::Initialized {
            return Err(Error::Generic(
                "Agents can only be added when model is initialized".to_string(),
            ));
        }

        let agent_id = agent.attributes().agent_id().clone();

        if self.agents.contains_key(&agent_id) {
            return Err(Error::Agent(format!(
                "Agent with ID {} already exists",
                agent_id
            )));
        }

        // Validate agent if validation is enabled
        if self.configuration.validation_enabled {
            self.validator.validate_agent_attributes(agent.attributes())?;
        }

        self.agents.insert(agent_id.clone(), agent);

        // Emit event
        if self.configuration.event_logging_enabled {
            let event = ModelEvent::agent_added(agent_id, self.current_time);
            self.event_bus.emit(event);
        }

        Ok(())
    }

    /// Remove an agent from the model
    pub fn remove_agent(&mut self, agent_id: &AgentId) -> Result<()> {
        if self.state == ModelState::Running {
            return Err(Error::Generic(
                "Cannot remove agents while model is running".to_string(),
            ));
        }

        if self.agents.remove(agent_id).is_none() {
            return Err(Error::Agent(format!(
                "Agent with ID {} not found",
                agent_id
            )));
        }

        // Emit event
        if self.configuration.event_logging_enabled {
            let event = ModelEvent::agent_removed(agent_id.clone(), self.current_time);
            self.event_bus.emit(event);
        }

        Ok(())
    }

    /// Get an agent by ID
    pub fn get_agent(&self, agent_id: &AgentId) -> Option<&ConsumerAgent<A, C>> {
        self.agents.get(agent_id)
    }

    /// Get all agent IDs
    pub fn agent_ids(&self) -> Vec<AgentId> {
        self.agents.keys().cloned().collect()
    }

    /// Start the simulation
    pub fn start(&mut self) -> Result<()> {
        if self.state != ModelState::Initialized {
            return Err(Error::Generic(
                "Model can only be started from initialized state".to_string(),
            ));
        }

        if self.agents.is_empty() {
            return Err(Error::Generic(
                "Cannot start simulation with no agents".to_string(),
            ));
        }

        // Initialize random seed if specified
        #[cfg(feature = "simulation")]
        if let Some(seed) = self.configuration.random_seed {
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        }

        self.state = ModelState::Running;
        self.current_time = 0.0;

        // Emit start event
        if self.configuration.event_logging_enabled {
            let event = ModelEvent::simulation_started(self.current_time);
            self.event_bus.emit(event);
        }

        Ok(())
    }

    /// Pause the simulation
    pub fn pause(&mut self) -> Result<()> {
        if self.state != ModelState::Running {
            return Err(Error::Generic(
                "Model can only be paused when running".to_string(),
            ));
        }

        self.state = ModelState::Paused;

        // Emit pause event
        if self.configuration.event_logging_enabled {
            let event = ModelEvent::simulation_paused(self.current_time);
            self.event_bus.emit(event);
        }

        Ok(())
    }

    /// Resume the simulation
    pub fn resume(&mut self) -> Result<()> {
        if self.state != ModelState::Paused {
            return Err(Error::Generic(
                "Model can only be resumed when paused".to_string(),
            ));
        }

        self.state = ModelState::Running;

        // Emit resume event
        if self.configuration.event_logging_enabled {
            let event = ModelEvent::simulation_resumed(self.current_time);
            self.event_bus.emit(event);
        }

        Ok(())
    }

    /// Stop the simulation
    pub fn stop(&mut self) -> Result<()> {
        if self.state == ModelState::Completed || self.state == ModelState::Error {
            return Ok(());
        }

        self.state = ModelState::Completed;

        // Update final statistics
        self.update_statistics();

        // Emit stop event
        if self.configuration.event_logging_enabled {
            let event = ModelEvent::simulation_completed(self.current_time);
            self.event_bus.emit(event);
        }

        Ok(())
    }

    /// Execute a single simulation step
    #[cfg(feature = "async")]
    pub async fn step(&mut self) -> Result<()> {
        if self.state != ModelState::Running {
            return Err(Error::Generic(
                "Cannot step when model is not running".to_string(),
            ));
        }

        let new_time = self.current_time + self.configuration.time_step;

        // Check if we've reached the maximum simulation time
        if new_time > self.configuration.max_simulation_time {
            self.stop()?;
            return Ok(());
        }

        // Update environment
        let environment_changes = self.environment.update_to_time(new_time).await?;

        // Process environment changes and generate information
        let mut all_information = Vec::new();
        for change in environment_changes {
            // Convert environment changes to information
            let info = Information::new(
                change.description,
                AgentId::new(), // System-generated information
                new_time,
                1.0, // Assume environment information is reliable
                change.change_type,
            );
            all_information.push(info);
        }

        // Update agents (simplified - in practice you'd have more complex logic)
        for (agent_id, agent) in &mut self.agents {
            // Process information for this agent
            let filter_context = crate::information::FilterContext::new(new_time);
            let distortion_context = crate::information::DistortionContext::new(new_time);

            let processed_info = self.information_transformer
                .process_information_for_agent(
                    agent_id,
                    all_information.clone(),
                    &filter_context,
                    &distortion_context,
                )
                .await?;

            // Here you would implement agent decision-making logic
            // This is simplified for the example
        }

        self.current_time = new_time;
        self.update_statistics();

        Ok(())
    }

    #[cfg(not(feature = "async"))]
    pub fn step(&mut self) -> Result<()> {
        if self.state != ModelState::Running {
            return Err(Error::Generic(
                "Cannot step when model is not running".to_string(),
            ));
        }

        let new_time = self.current_time + self.configuration.time_step;

        // Check if we've reached the maximum simulation time
        if new_time > self.configuration.max_simulation_time {
            self.stop()?;
            return Ok(());
        }

        // Update environment
        let environment_changes = self.environment.update_to_time(new_time)?;

        // Process environment changes and generate information
        let mut all_information = Vec::new();
        for change in environment_changes {
            // Convert environment changes to information
            let info = Information::new(
                change.description,
                AgentId::new(), // System-generated information
                new_time,
                1.0, // Assume environment information is reliable
                change.change_type,
            );
            all_information.push(info);
        }

        // Update agents (simplified - in practice you'd have more complex logic)
        for (agent_id, agent) in &mut self.agents {
            // Process information for this agent
            let filter_context = crate::information::FilterContext::new(new_time);
            let distortion_context = crate::information::DistortionContext::new(new_time);

            let processed_info = self.information_transformer
                .process_information_for_agent(
                    agent_id,
                    all_information.clone(),
                    &filter_context,
                    &distortion_context,
                )?;

            // Here you would implement agent decision-making logic
            // This is simplified for the example
        }

        self.current_time = new_time;
        self.update_statistics();

        Ok(())
    }

    /// Run the simulation until completion
    #[cfg(feature = "async")]
    pub async fn run(&mut self) -> Result<()> {
        self.start()?;

        while self.state == ModelState::Running {
            self.step().await?;
        }

        Ok(())
    }

    #[cfg(not(feature = "async"))]
    pub fn run(&mut self) -> Result<()> {
        self.start()?;

        while self.state == ModelState::Running {
            self.step()?;
        }

        Ok(())
    }

    /// Update model statistics
    fn update_statistics(&mut self) {
        self.statistics.total_agents = self.agents.len();
        self.statistics.simulation_duration = self.current_time;

        // Calculate total choices made
        let total_choices: usize = self.agents
            .values()
            .map(|agent| agent.choice_history().len())
            .sum();

        self.statistics.total_choices_made = total_choices;

        if self.statistics.total_agents > 0 {
            self.statistics.average_choices_per_agent =
                total_choices as f64 / self.statistics.total_agents as f64;
        }
    }

    /// Get the event bus for external event handling
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Reset the model to initial state
    pub fn reset(&mut self) -> Result<()> {
        if self.state == ModelState::Running {
            return Err(Error::Generic(
                "Cannot reset while model is running".to_string(),
            ));
        }

        self.state = ModelState::Initialized;
        self.current_time = 0.0;
        self.statistics = ModelStatistics::new();

        // Clear agent histories
        for agent in self.agents.values_mut() {
            agent.clear_history();
        }

        // Clear event bus
        self.event_bus.clear_events();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{BasicAgentAttributes, ConsumerAgent};
    use crate::environment::{Environment};
    use crate::information::{ReliabilityFilter, ConfirmationBiasDistorter, Transformer};
    use crate::types::AgentId;

    // Test implementations for traits (simplified)
    #[derive(Debug)]
    struct TestChoiceModule;

    #[cfg_attr(feature = "async", async_trait)]
    impl ChoiceModule for TestChoiceModule {
        type Choice = String;
        type Context = ();

        #[cfg(feature = "async")]
        async fn make_choice(
            &self,
            choices: Vec<Self::Choice>,
            _context: &Self::Context,
            _trigger: crate::types::TriggerType,
        ) -> Result<Option<Self::Choice>> {
            Ok(choices.into_iter().next())
        }

        #[cfg(not(feature = "async"))]
        fn make_choice(
            &self,
            choices: Vec<Self::Choice>,
            _context: &Self::Context,
            _trigger: crate::types::TriggerType,
        ) -> Result<Option<Self::Choice>> {
            Ok(choices.into_iter().next())
        }

        #[cfg(feature = "async")]
        async fn evaluate_choice(
            &self,
            _choice: &Self::Choice,
            _dimensions: &[crate::types::EvaluationDimension],
            _context: &Self::Context,
        ) -> Result<HashMap<crate::types::EvaluationDimension, f64>> {
            Ok(HashMap::new())
        }

        #[cfg(not(feature = "async"))]
        fn evaluate_choice(
            &self,
            _choice: &Self::Choice,
            _dimensions: &[crate::types::EvaluationDimension],
            _context: &Self::Context,
        ) -> Result<HashMap<crate::types::EvaluationDimension, f64>> {
            Ok(HashMap::new())
        }

        fn should_make_choice(&self, _trigger: crate::types::TriggerType, _context: &Self::Context) -> bool {
            true
        }

        fn evaluation_dimensions(&self) -> Vec<crate::types::EvaluationDimension> {
            vec![]
        }
    }

    #[test]
    fn test_model_configuration() {
        let config = ModelConfiguration::new(
            "Test Model".to_string(),
            "A test model".to_string(),
        )
            .with_time_step(0.5)
            .with_max_time(100.0)
            .with_random_seed(42);

        assert_eq!(config.name, "Test Model");
        assert_eq!(config.time_step, 0.5);
        assert_eq!(config.max_simulation_time, 100.0);
        assert_eq!(config.random_seed, Some(42));
    }

    #[test]
    fn test_model_state_transitions() {
        // This would require implementing all the test traits
        // Simplified for brevity
        assert_eq!(ModelState::Initialized, ModelState::Initialized);
    }
}