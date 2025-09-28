//! Basic usage example for the Consumer Choice Metamodel
//!
//! This example demonstrates how to:
//! - Create agent attributes and choice modules
//! - Set up a basic environment
//! - Create and configure a model
//! - Add agents to the model
//! - Run basic validation

use consumer_choice_metamodel::{
    agent::{AgentAttributes, BasicAgentAttributes, ChoiceModule, ConsumerAgent},
    environment::{Environment, PhysicalAsset, KnowledgeAsset, Network, RulesOfInteraction, ExogenousProcess},
    information::{ Transformer, ReliabilityFilter, ConfirmationBiasDistorter},
    model::{ConsumerChoiceModel, ModelConfiguration},
    types::{AgentId, AssetId, EvaluationDimension, SimulationTime, TriggerType},
    utils::{ModelValidator, PrintEventHandler},
    Result,
};
use std::collections::HashMap;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Simple choice representing a product to purchase
#[derive(Debug, Clone)]
struct ProductChoice {
    name: String,
    price: f64,
    quality: f64,
    environmental_impact: f64,
}

impl ProductChoice {
    fn new(name: String, price: f64, quality: f64, environmental_impact: f64) -> Self {
        Self {
            name,
            price,
            quality,
            environmental_impact,
        }
    }
}

/// Simple context for decision-making
#[derive(Debug)]
struct DecisionContext {
    available_budget: f64,
    time_pressure: f64,
    social_influence: f64,
}

impl DecisionContext {
    fn new(available_budget: f64) -> Self {
        Self {
            available_budget,
            time_pressure: 0.0,
            social_influence: 0.0,
        }
    }
}

/// Simple choice module that makes decisions based on agent attributes
#[derive(Debug)]
struct SimpleChoiceModule {
    evaluation_weights: HashMap<EvaluationDimension, f64>,
}

impl SimpleChoiceModule {
    fn new() -> Self {
        let mut evaluation_weights = HashMap::new();
        evaluation_weights.insert(EvaluationDimension::Economic, 0.4);
        evaluation_weights.insert(EvaluationDimension::Environmental, 0.3);
        evaluation_weights.insert(EvaluationDimension::Functional, 0.3);

        Self { evaluation_weights }
    }

    fn calculate_utility(&self, choice: &ProductChoice, attributes: &dyn AgentAttributes) -> f64 {
        let price_sensitivity = attributes
            .get_psychological_attribute("price_sensitivity")
            .unwrap_or(0.5);
        let environmental_concern = attributes
            .get_psychological_attribute("environmental_concern")
            .unwrap_or(0.5);
        let quality_preference = attributes
            .get_psychological_attribute("quality_preference")
            .unwrap_or(0.5);

        // Simple utility calculation
        let price_utility = 1.0 - (choice.price / 1000.0) * price_sensitivity;
        let environmental_utility = (1.0 - choice.environmental_impact) * environmental_concern;
        let quality_utility = choice.quality * quality_preference;

        let economic_weight = self.evaluation_weights.get(&EvaluationDimension::Economic).unwrap_or(&0.33);
        let environmental_weight = self.evaluation_weights.get(&EvaluationDimension::Environmental).unwrap_or(&0.33);
        let functional_weight = self.evaluation_weights.get(&EvaluationDimension::Functional).unwrap_or(&0.34);

        price_utility * economic_weight
            + environmental_utility * environmental_weight
            + quality_utility * functional_weight
    }
}

#[cfg_attr(feature = "async", async_trait)]
impl ChoiceModule for SimpleChoiceModule {
    type Choice = ProductChoice;
    type Context = DecisionContext;

    #[cfg(feature = "async")]
    async fn make_choice(
        &self,
        choices: Vec<Self::Choice>,
        context: &Self::Context,
        _trigger: TriggerType,
    ) -> Result<Option<Self::Choice>> {
        if choices.is_empty() {
            return Ok(None);
        }

        // Filter choices by budget
        let affordable_choices: Vec<_> = choices
            .into_iter()
            .filter(|choice| choice.price <= context.available_budget)
            .collect();

        if affordable_choices.is_empty() {
            return Ok(None);
        }

        // For simplicity, just return the first affordable choice
        // In a real implementation, you would calculate utilities and choose the best one
        Ok(affordable_choices.into_iter().next())
    }

    #[cfg(not(feature = "async"))]
    fn make_choice(
        &self,
        choices: Vec<Self::Choice>,
        context: &Self::Context,
        _trigger: TriggerType,
    ) -> Result<Option<Self::Choice>> {
        if choices.is_empty() {
            return Ok(None);
        }

        // Filter choices by budget
        let affordable_choices: Vec<_> = choices
            .into_iter()
            .filter(|choice| choice.price <= context.available_budget)
            .collect();

        if affordable_choices.is_empty() {
            return Ok(None);
        }

        // For simplicity, just return the first affordable choice
        // In a real implementation, you would calculate utilities and choose the best one
        Ok(affordable_choices.into_iter().next())
    }

    #[cfg(feature = "async")]
    async fn evaluate_choice(
        &self,
        choice: &Self::Choice,
        dimensions: &[EvaluationDimension],
        _context: &Self::Context,
    ) -> Result<HashMap<EvaluationDimension, f64>> {
        let mut scores = HashMap::new();

        for dimension in dimensions {
            let score = match dimension {
                EvaluationDimension::Economic => 1.0 - (choice.price / 1000.0),
                EvaluationDimension::Environmental => 1.0 - choice.environmental_impact,
                EvaluationDimension::Functional => choice.quality,
                _ => 0.5, // Default score for other dimensions
            };
            scores.insert(*dimension, score.clamp(0.0, 1.0));
        }

        Ok(scores)
    }

    #[cfg(not(feature = "async"))]
    fn evaluate_choice(
        &self,
        choice: &Self::Choice,
        dimensions: &[EvaluationDimension],
        _context: &Self::Context,
    ) -> Result<HashMap<EvaluationDimension, f64>> {
        let mut scores = HashMap::new();

        for dimension in dimensions {
            let score = match dimension {
                EvaluationDimension::Economic => 1.0 - (choice.price / 1000.0),
                EvaluationDimension::Environmental => 1.0 - choice.environmental_impact,
                EvaluationDimension::Functional => choice.quality,
                _ => 0.5, // Default score for other dimensions
            };
            scores.insert(dimension.clone(), score.clamp(0.0, 1.0));
        }

        Ok(scores)
    }

    fn should_make_choice(&self, trigger: TriggerType, context: &Self::Context) -> bool {
        match trigger {
            TriggerType::Economic => context.available_budget > 0.0,
            TriggerType::Informational => true,
            TriggerType::Social => context.social_influence > 0.5,
            _ => true,
        }
    }

    fn evaluation_dimensions(&self) -> Vec<EvaluationDimension> {
        self.evaluation_weights.keys().cloned().collect()
    }
}

// Minimal implementations for environment components
#[derive(Debug)]
struct DummyPhysicalAsset {
    id: AssetId,
    name: String,
}

impl PhysicalAsset for DummyPhysicalAsset {
    fn asset_id(&self) -> &AssetId { &self.id }
    fn name(&self) -> &str { &self.name }
    fn physical_properties(&self) -> HashMap<String, f64> { HashMap::new() }
    fn performance_characteristics(&self) -> HashMap<String, f64> { HashMap::new() }
    fn economic_attributes(&self) -> HashMap<String, f64> { HashMap::new() }
    fn environmental_impact(&self) -> HashMap<String, f64> { HashMap::new() }
    fn is_available(&self, _time: SimulationTime) -> bool { true }
    fn update_state(&mut self, _time: SimulationTime) -> Result<()> { Ok(()) }
}

#[derive(Debug)]
struct DummyKnowledgeAsset {
    id: AssetId,
    content: String,
}

impl KnowledgeAsset for DummyKnowledgeAsset {
    fn asset_id(&self) -> &AssetId { &self.id }
    fn content(&self) -> &str { &self.content }
    fn reliability(&self) -> f64 { 0.8 }
    fn relevance(&self, _topic: &str) -> f64 { 0.5 }
    fn timestamp(&self) -> SimulationTime { 0.0 }
    fn is_accessible_to(&self, _agent_id: &AgentId) -> bool { true }
    fn metadata(&self) -> HashMap<String, String> { HashMap::new() }
    fn update_reliability(&mut self, _new_reliability: f64) -> Result<()> { Ok(()) }
}

#[derive(Debug)]
struct DummyNetwork;

impl Network for DummyNetwork {
    fn agents(&self) -> Vec<AgentId> { Vec::new() }
    fn are_connected(&self, _agent1: &AgentId, _agent2: &AgentId) -> bool { false }
    fn connection_strength(&self, _agent1: &AgentId, _agent2: &AgentId) -> f64 { 0.0 }
    fn add_agent(&mut self, _agent_id: AgentId) -> Result<()> { Ok(()) }
    fn remove_agent(&mut self, _agent_id: &AgentId) -> Result<()> { Ok(()) }
    fn connect_agents(&mut self, _agent1: AgentId, _agent2: AgentId, _strength: f64) -> Result<()> { Ok(()) }
    fn neighbors(&self, _agent_id: &AgentId) -> Vec<AgentId> { Vec::new() }
    fn network_statistics(&self) -> consumer_choice_metamodel::environment::NetworkStatistics {
        consumer_choice_metamodel::environment::NetworkStatistics {
            agent_count: 0,
            connection_count: 0,
            average_degree: 0.0,
            clustering_coefficient: 0.0,
            network_density: 0.0,
        }
    }
}

#[derive(Debug)]
struct DummyRules;

#[cfg_attr(feature = "async", async_trait)]
impl RulesOfInteraction for DummyRules {
    type Interaction = String;

    #[cfg(feature = "async")]
    async fn is_interaction_allowed(&self, _initiator: &AgentId, _target: &AgentId, _interaction: &Self::Interaction, _time: SimulationTime) -> Result<bool> { Ok(true) }

    #[cfg(not(feature = "async"))]
    fn is_interaction_allowed(&self, _initiator: &AgentId, _target: &AgentId, _interaction: &Self::Interaction, _time: SimulationTime) -> Result<bool> { Ok(true) }

    #[cfg(feature = "async")]
    async fn process_interaction(&self, _initiator: &AgentId, target: &AgentId, _interaction: Self::Interaction, _time: SimulationTime) -> Result<Vec<consumer_choice_metamodel::environment::InteractionEffect>> {
        Ok(vec![consumer_choice_metamodel::environment::InteractionEffect {
            target_agent: target.clone(),
            effect_type: "dummy".to_string(),
            magnitude: 0.0,
            duration: None,
        }])
    }

    #[cfg(not(feature = "async"))]
    fn process_interaction(&self, _initiator: &AgentId, target: &AgentId, _interaction: Self::Interaction, _time: SimulationTime) -> Result<Vec<consumer_choice_metamodel::environment::InteractionEffect>> {
        Ok(vec![consumer_choice_metamodel::environment::InteractionEffect {
            target_agent: target.clone(),
            effect_type: "dummy".to_string(),
            magnitude: 0.0,
            duration: None,
        }])
    }

    fn interaction_cost(&self, _interaction: &Self::Interaction) -> f64 { 0.0 }
}

#[derive(Debug)]
struct DummyExogenousProcess;

#[cfg_attr(feature = "async", async_trait)]
impl ExogenousProcess for DummyExogenousProcess {
    #[cfg(feature = "async")]
    async fn update_environment(&self, _time: SimulationTime) -> Result<Vec<consumer_choice_metamodel::environment::EnvironmentChange>> { Ok(Vec::new()) }

    #[cfg(not(feature = "async"))]
    fn update_environment(&self, _time: SimulationTime) -> Result<Vec<consumer_choice_metamodel::environment::EnvironmentChange>> { Ok(Vec::new()) }

    fn is_active(&self, _time: SimulationTime) -> bool { false }
    fn name(&self) -> &str { "dummy" }
    fn frequency(&self) -> f64 { 0.0 }
}

fn main() -> Result<()> {
    println!("Consumer Choice Metamodel - Basic Usage Example");
    println!("==============================================");

    // 1. Create agent attributes
    println!("\n1. Creating agent attributes...");
    let agent_id = AgentId::new();
    let agent_attributes = BasicAgentAttributes::new(agent_id.clone())
        .with_psychological_attribute("price_sensitivity".to_string(), 0.7)
        .with_psychological_attribute("environmental_concern".to_string(), 0.8)
        .with_psychological_attribute("quality_preference".to_string(), 0.6)
        .with_socioeconomic_attribute("income".to_string(), 75000.0)
        .with_socioeconomic_attribute("age".to_string(), 35.0)
        .with_stock_variable("car".to_string(), Some("sedan".to_string()))
        .with_stock_variable("house".to_string(), None);

    println!("Created agent with ID: {}", agent_id);
    println!("Psychological attributes: {:?}", agent_attributes.psychological_attributes());
    println!("Socioeconomic attributes: {:?}", agent_attributes.socioeconomic_attributes());

    // 2. Create choice module
    println!("\n2. Creating choice module...");
    let choice_module = SimpleChoiceModule::new();
    println!("Choice module evaluation dimensions: {:?}", choice_module.evaluation_dimensions());

    // 3. Create consumer agent
    println!("\n3. Creating consumer agent...");
    let agent = ConsumerAgent::new(agent_attributes, choice_module);
    println!("Created consumer agent successfully");

    // 4. Validate agent
    println!("\n4. Validating agent...");
    let validator = ModelValidator::new();
    match validator.validate_agent_attributes(agent.attributes()) {
        Ok(()) => println!("Agent validation passed"),
        Err(e) => println!("Agent validation failed: {}", e),
    }

    // 5. Create environment components
    println!("\n5. Creating environment...");
    let environment: Environment<DummyPhysicalAsset, DummyKnowledgeAsset, DummyNetwork, DummyRules, DummyExogenousProcess> = Environment::new(DummyRules);
    println!("Environment created");

    // 6. Create information transformer
    println!("\n6. Creating information transformer...");
    let mut transformer: Transformer<ReliabilityFilter, ConfirmationBiasDistorter> = Transformer::new(100.0);
    transformer.add_filter(ReliabilityFilter::new(0.5));
    transformer.add_distorter(ConfirmationBiasDistorter::new(0.3));
    println!("Information transformer created with {} filters and {} distorters",
             transformer.filter_count(), transformer.distorter_count());

    // 7. Create model configuration
    println!("\n7. Creating model configuration...");
    let config = ModelConfiguration::new(
        "Basic Usage Example".to_string(),
        "A simple example demonstrating basic usage".to_string(),
    )
        .with_time_step(1.0)
        .with_max_time(10.0)
        .with_random_seed(42)
        .with_validation(true);

    println!("Model configuration created: {}", config.name);

    // 8. Create model
    println!("\n8. Creating model...");
    let mut model: ConsumerChoiceModel<
        BasicAgentAttributes,
        SimpleChoiceModule,
        DummyPhysicalAsset,
        DummyKnowledgeAsset,
        DummyNetwork,
        DummyRules,
        DummyExogenousProcess,
        ReliabilityFilter,
        ConfirmationBiasDistorter,
    > = ConsumerChoiceModel::new(config, environment, transformer);
    println!("Model created with ID: {}", model.configuration().model_id);

    // 9. Add event handler
    println!("\n9. Setting up event handling...");
    model.event_bus().add_handler(Box::new(PrintEventHandler));
    println!("Event handler added");

    // 10. Add agent to model
    println!("\n10. Adding agent to model...");
    match model.add_agent(agent) {
        Ok(()) => println!("Agent added successfully"),
        Err(e) => println!("Failed to add agent: {}", e),
    }

    // 11. Display model statistics
    println!("\n11. Model statistics:");
    let stats = model.statistics();
    println!("   Total agents: {}", stats.total_agents);
    println!("   Total choices made: {}", stats.total_choices_made);
    println!("   Simulation duration: {:.2}", stats.simulation_duration);

    // 12. Test choice making (simplified example)
    println!("\n12. Testing choice making...");
    let choices = vec![
        ProductChoice::new("Eco-friendly Car".to_string(), 30000.0, 0.8, 0.2),
        ProductChoice::new("Economy Car".to_string(), 20000.0, 0.6, 0.6),
        ProductChoice::new("Luxury Car".to_string(), 50000.0, 0.9, 0.8),
    ];

    let context = DecisionContext::new(35000.0);

    // Get the agent and test choice making
    let agent_ids = model.agent_ids();
    if let Some(agent_id) = agent_ids.first() {
        if let Some(agent) = model.get_agent(agent_id) {
            #[cfg(not(feature = "async"))]
            {
                match agent.choice_module().make_choice(choices, &context, TriggerType::Economic) {
                    Ok(Some(choice)) => println!("Agent chose: {} (${:.2})", choice.name, choice.price),
                    Ok(None) => println!("Agent made no choice"),
                    Err(e) => println!("Choice making failed: {}", e),
                }
            }

            #[cfg(feature = "async")]
            {
                println!("Choice making test requires async runtime (use 'async' feature)");
            }
        }
    }

    println!("\n==============================================");
    println!("Basic usage example completed successfully!");
    println!("==============================================");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_choice_creation() {
        let choice = ProductChoice::new("Test Product".to_string(), 100.0, 0.8, 0.3);
        assert_eq!(choice.name, "Test Product");
        assert_eq!(choice.price, 100.0);
    }

    #[test]
    fn test_simple_choice_module() {
        let module = SimpleChoiceModule::new();
        assert_eq!(module.evaluation_dimensions().len(), 3);
    }

    #[test]
    fn test_decision_context() {
        let context = DecisionContext::new(1000.0);
        assert_eq!(context.available_budget, 1000.0);
        assert_eq!(context.time_pressure, 0.0);
    }
}