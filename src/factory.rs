//! Factory pattern implementations for the Consumer Choice Metamodel

use crate::agent::{AgentAttributes, ChoiceModule, ConsumerAgent};
use crate::environment::{
    ExogenousProcess, KnowledgeAsset, Network, PhysicalAsset, RulesOfInteraction,
};
use crate::information::{InformationDistorter, InformationFilter};
use crate::types::{AgentId, AssetId, SimulationTime};
use crate::{Error, Result};
use std::collections::HashMap;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Configuration for creating agents
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AgentConfig {
    pub psychological_attributes: HashMap<String, f64>,
    pub socioeconomic_attributes: HashMap<String, f64>,
    pub stock_variables: HashMap<String, Option<String>>,
    pub choice_module_params: HashMap<String, f64>,
}

impl AgentConfig {
    pub fn new() -> Self {
        Self {
            psychological_attributes: HashMap::new(),
            socioeconomic_attributes: HashMap::new(),
            stock_variables: HashMap::new(),
            choice_module_params: HashMap::new(),
        }
    }

    pub fn with_psychological_attribute(mut self, name: String, value: f64) -> Self {
        self.psychological_attributes.insert(name, value);
        self
    }

    pub fn with_socioeconomic_attribute(mut self, name: String, value: f64) -> Self {
        self.socioeconomic_attributes.insert(name, value);
        self
    }

    pub fn with_stock_variable(mut self, name: String, value: Option<String>) -> Self {
        self.stock_variables.insert(name, value);
        self
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for creating physical assets
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PhysicalAssetConfig {
    pub name: String,
    pub physical_properties: HashMap<String, f64>,
    pub performance_characteristics: HashMap<String, f64>,
    pub economic_attributes: HashMap<String, f64>,
    pub environmental_impact: HashMap<String, f64>,
    pub availability_start: SimulationTime,
    pub availability_end: Option<SimulationTime>,
}

impl PhysicalAssetConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            physical_properties: HashMap::new(),
            performance_characteristics: HashMap::new(),
            economic_attributes: HashMap::new(),
            environmental_impact: HashMap::new(),
            availability_start: 0.0,
            availability_end: None,
        }
    }
}

/// Configuration for creating knowledge assets
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KnowledgeAssetConfig {
    pub content: String,
    pub reliability: f64,
    pub topic: String,
    pub metadata: HashMap<String, String>,
    pub access_agents: Vec<AgentId>,
}

impl KnowledgeAssetConfig {
    pub fn new(content: String, topic: String) -> Self {
        Self {
            content,
            reliability: 1.0,
            topic,
            metadata: HashMap::new(),
            access_agents: Vec::new(),
        }
    }
}

/// Trait for creating model components using the factory pattern
#[cfg_attr(feature = "async", async_trait)]
pub trait ModelComponentFactory: std::fmt::Debug + Send + Sync {
    /// Type of agents this factory creates
    type Agent: AgentAttributes + 'static;

    /// Type of choice modules this factory creates
    type ChoiceModule: ChoiceModule + 'static;

    /// Type of physical assets this factory creates
    type PhysicalAsset: PhysicalAsset;

    /// Type of knowledge assets this factory creates
    type KnowledgeAsset: KnowledgeAsset;

    /// Type of networks this factory creates
    type Network: Network;

    /// Type of interaction rules this factory creates
    type InteractionRules: RulesOfInteraction;

    /// Type of exogenous processes this factory creates
    type ExogenousProcess: ExogenousProcess;

    /// Type of information filters this factory creates
    type InformationFilter: InformationFilter;

    /// Type of information distorters this factory creates
    type InformationDistorter: InformationDistorter;

    /// Create an agent with the specified configuration
    #[cfg(feature = "async")]
    async fn create_agent(
        &self,
        agent_id: AgentId,
        config: &AgentConfig,
    ) -> Result<ConsumerAgent<Self::Agent, Self::ChoiceModule>>;

    #[cfg(not(feature = "async"))]
    fn create_agent(
        &self,
        agent_id: AgentId,
        config: &AgentConfig,
    ) -> Result<ConsumerAgent<Self::Agent, Self::ChoiceModule>>;

    /// Create a physical asset
    #[cfg(feature = "async")]
    async fn create_physical_asset(
        &self,
        asset_id: AssetId,
        config: &PhysicalAssetConfig,
    ) -> Result<Self::PhysicalAsset>;

    #[cfg(not(feature = "async"))]
    fn create_physical_asset(
        &self,
        asset_id: AssetId,
        config: &PhysicalAssetConfig,
    ) -> Result<Self::PhysicalAsset>;

    /// Create a knowledge asset
    #[cfg(feature = "async")]
    async fn create_knowledge_asset(
        &self,
        asset_id: AssetId,
        config: &KnowledgeAssetConfig,
    ) -> Result<Self::KnowledgeAsset>;

    #[cfg(not(feature = "async"))]
    fn create_knowledge_asset(
        &self,
        asset_id: AssetId,
        config: &KnowledgeAssetConfig,
    ) -> Result<Self::KnowledgeAsset>;

    /// Create a network
    #[cfg(feature = "async")]
    async fn create_network(&self) -> Result<Self::Network>;

    #[cfg(not(feature = "async"))]
    fn create_network(&self) -> Result<Self::Network>;

    /// Create interaction rules
    #[cfg(feature = "async")]
    async fn create_interaction_rules(&self) -> Result<Self::InteractionRules>;

    #[cfg(not(feature = "async"))]
    fn create_interaction_rules(&self) -> Result<Self::InteractionRules>;

    /// Create an exogenous process
    #[cfg(feature = "async")]
    async fn create_exogenous_process(&self, name: String) -> Result<Self::ExogenousProcess>;

    #[cfg(not(feature = "async"))]
    fn create_exogenous_process(&self, name: String) -> Result<Self::ExogenousProcess>;

    /// Create an information filter
    #[cfg(feature = "async")]
    async fn create_information_filter(&self, filter_type: &str)
        -> Result<Self::InformationFilter>;

    #[cfg(not(feature = "async"))]
    fn create_information_filter(&self, filter_type: &str) -> Result<Self::InformationFilter>;

    /// Create an information distorter
    #[cfg(feature = "async")]
    async fn create_information_distorter(
        &self,
        distorter_type: &str,
    ) -> Result<Self::InformationDistorter>;

    #[cfg(not(feature = "async"))]
    fn create_information_distorter(
        &self,
        distorter_type: &str,
    ) -> Result<Self::InformationDistorter>;

    /// Get factory name
    fn factory_name(&self) -> &str;

    /// Get supported component types
    fn supported_component_types(&self) -> Vec<String>;
}

/// Basic factory implementation for creating standard components
#[derive(Debug)]
pub struct BasicModelFactory {
    name: String,
}

impl BasicModelFactory {
    /// Create a new basic factory
    pub fn new() -> Self {
        Self {
            name: "BasicModelFactory".to_string(),
        }
    }

    /// Create a factory with a custom name
    pub fn with_name(name: String) -> Self {
        Self { name }
    }
}

impl Default for BasicModelFactory {
    fn default() -> Self {
        Self::new()
    }
}

// For the basic factory, we need to define the associated types
// In a real implementation, you would implement this for your specific types
#[cfg(test)]
mod factory_tests {
    use super::*;
    use crate::agent::BasicAgentAttributes;
    use crate::information::{ConfirmationBiasDistorter, ReliabilityFilter};

    // Mock implementations for testing
    #[derive(Debug)]
    struct MockChoiceModule;

    #[cfg_attr(feature = "async", async_trait)]
    impl ChoiceModule for MockChoiceModule {
        type Choice = String;
        type Context = ();

        #[cfg(feature = "async")]
        async fn make_choice(
            &self,
            choices: Vec<String>,
            _context: &(),
            _trigger: crate::types::TriggerType,
        ) -> Result<Option<String>> {
            Ok(choices.into_iter().next())
        }

        #[cfg(not(feature = "async"))]
        fn make_choice(
            &self,
            choices: Vec<String>,
            _context: &(),
            _trigger: crate::types::TriggerType,
        ) -> Result<Option<String>> {
            Ok(choices.into_iter().next())
        }

        #[cfg(feature = "async")]
        async fn evaluate_choice(
            &self,
            _choice: &String,
            _dimensions: &[crate::types::EvaluationDimension],
            _context: &(),
        ) -> Result<HashMap<crate::types::EvaluationDimension, f64>> {
            Ok(HashMap::new())
        }

        #[cfg(not(feature = "async"))]
        fn evaluate_choice(
            &self,
            _choice: &String,
            _dimensions: &[crate::types::EvaluationDimension],
            _context: &(),
        ) -> Result<HashMap<crate::types::EvaluationDimension, f64>> {
            Ok(HashMap::new())
        }

        fn should_make_choice(&self, _trigger: crate::types::TriggerType, _context: &()) -> bool {
            true
        }
        fn evaluation_dimensions(&self) -> Vec<crate::types::EvaluationDimension> {
            Vec::new()
        }
    }

    #[derive(Debug)]
    struct MockPhysicalAsset {
        id: AssetId,
        name: String,
    }

    impl PhysicalAsset for MockPhysicalAsset {
        fn asset_id(&self) -> &AssetId {
            &self.id
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn physical_properties(&self) -> HashMap<String, f64> {
            HashMap::new()
        }
        fn performance_characteristics(&self) -> HashMap<String, f64> {
            HashMap::new()
        }
        fn economic_attributes(&self) -> HashMap<String, f64> {
            HashMap::new()
        }
        fn environmental_impact(&self) -> HashMap<String, f64> {
            HashMap::new()
        }
        fn is_available(&self, _time: SimulationTime) -> bool {
            true
        }
        fn update_state(&mut self, _time: SimulationTime) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct MockKnowledgeAsset {
        id: AssetId,
        content: String,
    }

    impl KnowledgeAsset for MockKnowledgeAsset {
        fn asset_id(&self) -> &AssetId {
            &self.id
        }
        fn content(&self) -> &str {
            &self.content
        }
        fn reliability(&self) -> f64 {
            0.8
        }
        fn relevance(&self, _topic: &str) -> f64 {
            0.5
        }
        fn timestamp(&self) -> SimulationTime {
            0.0
        }
        fn is_accessible_to(&self, _agent_id: &AgentId) -> bool {
            true
        }
        fn metadata(&self) -> HashMap<String, String> {
            HashMap::new()
        }
        fn update_reliability(&mut self, _new_reliability: f64) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct MockNetwork;

    impl Network for MockNetwork {
        fn agents(&self) -> Vec<AgentId> {
            Vec::new()
        }
        fn are_connected(&self, _agent1: &AgentId, _agent2: &AgentId) -> bool {
            false
        }
        fn connection_strength(&self, _agent1: &AgentId, _agent2: &AgentId) -> f64 {
            0.0
        }
        fn add_agent(&mut self, _agent_id: AgentId) -> Result<()> {
            Ok(())
        }
        fn remove_agent(&mut self, _agent_id: &AgentId) -> Result<()> {
            Ok(())
        }
        fn connect_agents(
            &mut self,
            _agent1: AgentId,
            _agent2: AgentId,
            _strength: f64,
        ) -> Result<()> {
            Ok(())
        }
        fn neighbors(&self, _agent_id: &AgentId) -> Vec<AgentId> {
            Vec::new()
        }
        fn network_statistics(&self) -> crate::environment::NetworkStatistics {
            crate::environment::NetworkStatistics {
                agent_count: 0,
                connection_count: 0,
                average_degree: 0.0,
                clustering_coefficient: 0.0,
                network_density: 0.0,
            }
        }
    }

    #[derive(Debug)]
    struct MockRules;

    #[cfg_attr(feature = "async", async_trait)]
    impl RulesOfInteraction for MockRules {
        type Interaction = String;

        #[cfg(feature = "async")]
        async fn is_interaction_allowed(
            &self,
            _initiator: &AgentId,
            _target: &AgentId,
            _interaction: &String,
            _time: SimulationTime,
        ) -> Result<bool> {
            Ok(true)
        }

        #[cfg(not(feature = "async"))]
        fn is_interaction_allowed(
            &self,
            _initiator: &AgentId,
            _target: &AgentId,
            _interaction: &String,
            _time: SimulationTime,
        ) -> Result<bool> {
            Ok(true)
        }

        #[cfg(feature = "async")]
        async fn process_interaction(
            &self,
            _initiator: &AgentId,
            target: &AgentId,
            _interaction: String,
            _time: SimulationTime,
        ) -> Result<Vec<crate::environment::InteractionEffect>> {
            Ok(vec![crate::environment::InteractionEffect {
                target_agent: target.clone(),
                effect_type: "test".to_string(),
                magnitude: 0.0,
                duration: None,
            }])
        }

        #[cfg(not(feature = "async"))]
        fn process_interaction(
            &self,
            _initiator: &AgentId,
            target: &AgentId,
            _interaction: String,
            _time: SimulationTime,
        ) -> Result<Vec<crate::environment::InteractionEffect>> {
            Ok(vec![crate::environment::InteractionEffect {
                target_agent: target.clone(),
                effect_type: "test".to_string(),
                magnitude: 0.0,
                duration: None,
            }])
        }

        fn interaction_cost(&self, _interaction: &String) -> f64 {
            0.0
        }
    }

    #[derive(Debug)]
    struct MockExogenousProcess;

    #[cfg_attr(feature = "async", async_trait)]
    impl ExogenousProcess for MockExogenousProcess {
        #[cfg(feature = "async")]
        async fn update_environment(
            &self,
            _time: SimulationTime,
        ) -> Result<Vec<crate::environment::EnvironmentChange>> {
            Ok(Vec::new())
        }

        #[cfg(not(feature = "async"))]
        fn update_environment(
            &self,
            _time: SimulationTime,
        ) -> Result<Vec<crate::environment::EnvironmentChange>> {
            Ok(Vec::new())
        }

        fn is_active(&self, _time: SimulationTime) -> bool {
            false
        }
        fn name(&self) -> &str {
            "mock"
        }
        fn frequency(&self) -> f64 {
            0.0
        }
    }

    #[derive(Debug)]
    struct TestFactory;

    #[cfg_attr(feature = "async", async_trait)]
    impl ModelComponentFactory for TestFactory {
        type Agent = BasicAgentAttributes;
        type ChoiceModule = MockChoiceModule;
        type PhysicalAsset = MockPhysicalAsset;
        type KnowledgeAsset = MockKnowledgeAsset;
        type Network = MockNetwork;
        type InteractionRules = MockRules;
        type ExogenousProcess = MockExogenousProcess;
        type InformationFilter = ReliabilityFilter;
        type InformationDistorter = ConfirmationBiasDistorter;

        #[cfg(feature = "async")]
        async fn create_agent(
            &self,
            agent_id: AgentId,
            config: &AgentConfig,
        ) -> Result<crate::agent::ConsumerAgent<Self::Agent, Self::ChoiceModule>> {
            let mut attributes = BasicAgentAttributes::new(agent_id);
            for (name, value) in &config.psychological_attributes {
                attributes = attributes.with_psychological_attribute(name.clone(), *value);
            }
            for (name, value) in &config.socioeconomic_attributes {
                attributes = attributes.with_socioeconomic_attribute(name.clone(), *value);
            }

            let choice_module = MockChoiceModule;
            Ok(crate::agent::ConsumerAgent::new(attributes, choice_module))
        }

        #[cfg(not(feature = "async"))]
        fn create_agent(
            &self,
            agent_id: AgentId,
            config: &AgentConfig,
        ) -> Result<crate::agent::ConsumerAgent<Self::Agent, Self::ChoiceModule>> {
            let mut attributes = BasicAgentAttributes::new(agent_id);
            for (name, value) in &config.psychological_attributes {
                attributes = attributes.with_psychological_attribute(name.clone(), *value);
            }
            for (name, value) in &config.socioeconomic_attributes {
                attributes = attributes.with_socioeconomic_attribute(name.clone(), *value);
            }

            let choice_module = MockChoiceModule;
            Ok(crate::agent::ConsumerAgent::new(attributes, choice_module))
        }

        #[cfg(feature = "async")]
        async fn create_physical_asset(
            &self,
            asset_id: AssetId,
            config: &PhysicalAssetConfig,
        ) -> Result<Self::PhysicalAsset> {
            Ok(MockPhysicalAsset {
                id: asset_id,
                name: config.name.clone(),
            })
        }

        #[cfg(not(feature = "async"))]
        fn create_physical_asset(
            &self,
            asset_id: AssetId,
            config: &PhysicalAssetConfig,
        ) -> Result<Self::PhysicalAsset> {
            Ok(MockPhysicalAsset {
                id: asset_id,
                name: config.name.clone(),
            })
        }

        #[cfg(feature = "async")]
        async fn create_knowledge_asset(
            &self,
            asset_id: AssetId,
            config: &KnowledgeAssetConfig,
        ) -> Result<Self::KnowledgeAsset> {
            Ok(MockKnowledgeAsset {
                id: asset_id,
                content: config.content.clone(),
            })
        }

        #[cfg(not(feature = "async"))]
        fn create_knowledge_asset(
            &self,
            asset_id: AssetId,
            config: &KnowledgeAssetConfig,
        ) -> Result<Self::KnowledgeAsset> {
            Ok(MockKnowledgeAsset {
                id: asset_id,
                content: config.content.clone(),
            })
        }

        #[cfg(feature = "async")]
        async fn create_network(&self) -> Result<Self::Network> {
            Ok(MockNetwork)
        }

        #[cfg(not(feature = "async"))]
        fn create_network(&self) -> Result<Self::Network> {
            Ok(MockNetwork)
        }

        #[cfg(feature = "async")]
        async fn create_interaction_rules(&self) -> Result<Self::InteractionRules> {
            Ok(MockRules)
        }

        #[cfg(not(feature = "async"))]
        fn create_interaction_rules(&self) -> Result<Self::InteractionRules> {
            Ok(MockRules)
        }

        #[cfg(feature = "async")]
        async fn create_exogenous_process(&self, _name: String) -> Result<Self::ExogenousProcess> {
            Ok(MockExogenousProcess)
        }

        #[cfg(not(feature = "async"))]
        fn create_exogenous_process(&self, _name: String) -> Result<Self::ExogenousProcess> {
            Ok(MockExogenousProcess)
        }

        #[cfg(feature = "async")]
        async fn create_information_filter(
            &self,
            _filter_type: &str,
        ) -> Result<Self::InformationFilter> {
            Ok(ReliabilityFilter::new(0.5))
        }

        #[cfg(not(feature = "async"))]
        fn create_information_filter(&self, _filter_type: &str) -> Result<Self::InformationFilter> {
            Ok(ReliabilityFilter::new(0.5))
        }

        #[cfg(feature = "async")]
        async fn create_information_distorter(
            &self,
            _distorter_type: &str,
        ) -> Result<Self::InformationDistorter> {
            Ok(ConfirmationBiasDistorter::new(0.3))
        }

        #[cfg(not(feature = "async"))]
        fn create_information_distorter(
            &self,
            _distorter_type: &str,
        ) -> Result<Self::InformationDistorter> {
            Ok(ConfirmationBiasDistorter::new(0.3))
        }

        fn factory_name(&self) -> &str {
            "TestFactory"
        }

        fn supported_component_types(&self) -> Vec<String> {
            vec![
                "BasicAgent".to_string(),
                "MockPhysicalAsset".to_string(),
                "MockKnowledgeAsset".to_string(),
                "MockNetwork".to_string(),
            ]
        }
    }

    #[test]
    fn test_agent_config() {
        let config = AgentConfig::new()
            .with_psychological_attribute("risk_aversion".to_string(), 0.5)
            .with_socioeconomic_attribute("income".to_string(), 50000.0);

        assert_eq!(
            config.psychological_attributes.get("risk_aversion"),
            Some(&0.5)
        );
        assert_eq!(
            config.socioeconomic_attributes.get("income"),
            Some(&50000.0)
        );
    }

    #[test]
    fn test_physical_asset_config() {
        let config = PhysicalAssetConfig::new("Test Asset".to_string());
        assert_eq!(config.name, "Test Asset");
        assert_eq!(config.availability_start, 0.0);
    }

    #[test]
    fn test_knowledge_asset_config() {
        let config =
            KnowledgeAssetConfig::new("Test content".to_string(), "test_topic".to_string());
        assert_eq!(config.content, "Test content");
        assert_eq!(config.topic, "test_topic");
        assert_eq!(config.reliability, 1.0);
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn test_factory_agent_creation() {
        let factory = TestFactory;
        let agent_id = AgentId::new();
        let config =
            AgentConfig::new().with_psychological_attribute("risk_aversion".to_string(), 0.7);

        let result = factory.create_agent(agent_id.clone(), &config);
        assert!(result.is_ok());

        let agent = result.unwrap();
        assert_eq!(agent.attributes().agent_id(), &agent_id);
        assert_eq!(
            agent
                .attributes()
                .get_psychological_attribute("risk_aversion"),
            Some(0.7)
        );
    }

    #[test]
    fn test_factory_info() {
        let factory = TestFactory;
        assert_eq!(factory.factory_name(), "TestFactory");
        assert!(!factory.supported_component_types().is_empty());
    }
}
