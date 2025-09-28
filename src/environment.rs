//! Environment and asset-related traits and types for the Consumer Choice Metamodel

use crate::types::{AgentId, AssetId, SimulationTime};
use crate::{Error, Result};
use std::collections::HashMap;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Trait for physical assets/technologies in the environment
pub trait PhysicalAsset: std::fmt::Debug + Send + Sync {
    /// Get the unique identifier for this asset
    fn asset_id(&self) -> &AssetId;

    /// Get the name/description of this asset
    fn name(&self) -> &str;

    /// Get physical properties of the asset (e.g., size, weight, capacity)
    fn physical_properties(&self) -> HashMap<String, f64>;

    /// Get performance characteristics (e.g., efficiency, durability)
    fn performance_characteristics(&self) -> HashMap<String, f64>;

    /// Get economic attributes (e.g., price, maintenance cost)
    fn economic_attributes(&self) -> HashMap<String, f64>;

    /// Get environmental impact metrics
    fn environmental_impact(&self) -> HashMap<String, f64>;

    /// Check if the asset is available at the given time
    fn is_available(&self, time: SimulationTime) -> bool;

    /// Update asset state based on usage or time passage
    fn update_state(&mut self, time: SimulationTime) -> Result<()>;

    /// Get a specific physical property by name
    fn get_physical_property(&self, name: &str) -> Option<f64> {
        self.physical_properties().get(name).copied()
    }

    /// Get a specific performance characteristic by name
    fn get_performance_characteristic(&self, name: &str) -> Option<f64> {
        self.performance_characteristics().get(name).copied()
    }
}

/// Trait for knowledge/information assets in the environment
pub trait KnowledgeAsset: std::fmt::Debug + Send + Sync {
    /// Get the unique identifier for this knowledge asset
    fn asset_id(&self) -> &AssetId;

    /// Get the content/information this asset represents
    fn content(&self) -> &str;

    /// Get the reliability/credibility of this information (0.0 to 1.0)
    fn reliability(&self) -> f64;

    /// Get the relevance to specific topics or agents
    fn relevance(&self, topic: &str) -> f64;

    /// Get the timestamp when this information was created/last updated
    fn timestamp(&self) -> SimulationTime;

    /// Check if this information is accessible to a specific agent
    fn is_accessible_to(&self, agent_id: &AgentId) -> bool;

    /// Get associated metadata
    fn metadata(&self) -> HashMap<String, String>;

    /// Update the reliability based on validation or time passage
    fn update_reliability(&mut self, new_reliability: f64) -> Result<()>;
}

/// Trait for networks connecting agents
pub trait Network: std::fmt::Debug + Send + Sync {
    /// Get all agents connected in this network
    fn agents(&self) -> Vec<AgentId>;

    /// Check if two agents are connected
    fn are_connected(&self, agent1: &AgentId, agent2: &AgentId) -> bool;

    /// Get the strength of connection between two agents (0.0 to 1.0)
    fn connection_strength(&self, agent1: &AgentId, agent2: &AgentId) -> f64;

    /// Add an agent to the network
    fn add_agent(&mut self, agent_id: AgentId) -> Result<()>;

    /// Remove an agent from the network
    fn remove_agent(&mut self, agent_id: &AgentId) -> Result<()>;

    /// Create or update a connection between two agents
    fn connect_agents(&mut self, agent1: AgentId, agent2: AgentId, strength: f64) -> Result<()>;

    /// Get the neighbors of a specific agent
    fn neighbors(&self, agent_id: &AgentId) -> Vec<AgentId>;

    /// Get network statistics
    fn network_statistics(&self) -> NetworkStatistics;
}

/// Network statistics for analysis
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NetworkStatistics {
    pub agent_count: usize,
    pub connection_count: usize,
    pub average_degree: f64,
    pub clustering_coefficient: f64,
    pub network_density: f64,
}

/// Trait for rules governing agent interactions
#[cfg_attr(feature = "async", async_trait)]
pub trait RulesOfInteraction: std::fmt::Debug + Send + Sync {
    /// Type representing an interaction between agents
    type Interaction: std::fmt::Debug + Send + Sync;

    /// Check if an interaction between two agents is allowed
    #[cfg(feature = "async")]
    async fn is_interaction_allowed(
        &self,
        initiator: &AgentId,
        target: &AgentId,
        interaction: &Self::Interaction,
        time: SimulationTime,
    ) -> Result<bool>;

    #[cfg(not(feature = "async"))]
    fn is_interaction_allowed(
        &self,
        initiator: &AgentId,
        target: &AgentId,
        interaction: &Self::Interaction,
        time: SimulationTime,
    ) -> Result<bool>;

    /// Process an interaction and return any resulting effects
    #[cfg(feature = "async")]
    async fn process_interaction(
        &self,
        initiator: &AgentId,
        target: &AgentId,
        interaction: Self::Interaction,
        time: SimulationTime,
    ) -> Result<Vec<InteractionEffect>>;

    #[cfg(not(feature = "async"))]
    fn process_interaction(
        &self,
        initiator: &AgentId,
        target: &AgentId,
        interaction: Self::Interaction,
        time: SimulationTime,
    ) -> Result<Vec<InteractionEffect>>;

    /// Get the cost/effort required for an interaction
    fn interaction_cost(&self, interaction: &Self::Interaction) -> f64;
}

/// Effect resulting from an agent interaction
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InteractionEffect {
    pub target_agent: AgentId,
    pub effect_type: String,
    pub magnitude: f64,
    pub duration: Option<SimulationTime>,
}

/// Trait for exogenous processes affecting the environment
#[cfg_attr(feature = "async", async_trait)]
pub trait ExogenousProcess: std::fmt::Debug + Send + Sync {
    /// Update the environment based on this exogenous process
    #[cfg(feature = "async")]
    async fn update_environment(&self, time: SimulationTime) -> Result<Vec<EnvironmentChange>>;

    #[cfg(not(feature = "async"))]
    fn update_environment(&self, time: SimulationTime) -> Result<Vec<EnvironmentChange>>;

    /// Check if this process is active at the given time
    fn is_active(&self, time: SimulationTime) -> bool;

    /// Get the name/description of this process
    fn name(&self) -> &str;

    /// Get the frequency of this process (how often it occurs)
    fn frequency(&self) -> f64;
}

/// Change to the environment from an exogenous process
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnvironmentChange {
    pub change_type: String,
    pub affected_assets: Vec<AssetId>,
    pub magnitude: f64,
    pub duration: Option<SimulationTime>,
    pub description: String,
}

/// Main environment container class
#[derive(Debug)]
pub struct Environment<P, K, N, R, E>
where
    P: PhysicalAsset,
    K: KnowledgeAsset,
    N: Network,
    R: RulesOfInteraction,
    E: ExogenousProcess,
{
    physical_assets: HashMap<AssetId, P>,
    knowledge_assets: HashMap<AssetId, K>,
    networks: Vec<N>,
    interaction_rules: R,
    exogenous_processes: Vec<E>,
    current_time: SimulationTime,
}

impl<P, K, N, R, E> Environment<P, K, N, R, E>
where
    P: PhysicalAsset,
    K: KnowledgeAsset,
    N: Network,
    R: RulesOfInteraction,
    E: ExogenousProcess,
{
    /// Create a new environment
    pub fn new(interaction_rules: R) -> Self {
        Self {
            physical_assets: HashMap::new(),
            knowledge_assets: HashMap::new(),
            networks: Vec::new(),
            interaction_rules,
            exogenous_processes: Vec::new(),
            current_time: 0.0,
        }
    }

    /// Add a physical asset to the environment
    pub fn add_physical_asset(&mut self, asset: P) -> Result<()> {
        let asset_id = asset.asset_id().clone();
        if self.physical_assets.contains_key(&asset_id) {
            return Err(Error::Environment(format!(
                "Physical asset with ID {} already exists",
                asset_id
            )));
        }
        self.physical_assets.insert(asset_id, asset);
        Ok(())
    }

    /// Add a knowledge asset to the environment
    pub fn add_knowledge_asset(&mut self, asset: K) -> Result<()> {
        let asset_id = asset.asset_id().clone();
        if self.knowledge_assets.contains_key(&asset_id) {
            return Err(Error::Environment(format!(
                "Knowledge asset with ID {} already exists",
                asset_id
            )));
        }
        self.knowledge_assets.insert(asset_id, asset);
        Ok(())
    }

    /// Add a network to the environment
    pub fn add_network(&mut self, network: N) {
        self.networks.push(network);
    }

    /// Add an exogenous process
    pub fn add_exogenous_process(&mut self, process: E) {
        self.exogenous_processes.push(process);
    }

    /// Get a physical asset by ID
    pub fn get_physical_asset(&self, asset_id: &AssetId) -> Option<&P> {
        self.physical_assets.get(asset_id)
    }

    /// Get a knowledge asset by ID
    pub fn get_knowledge_asset(&self, asset_id: &AssetId) -> Option<&K> {
        self.knowledge_assets.get(asset_id)
    }

    /// Get all physical assets
    pub fn physical_assets(&self) -> impl Iterator<Item = &P> {
        self.physical_assets.values()
    }

    /// Get all knowledge assets
    pub fn knowledge_assets(&self) -> impl Iterator<Item = &K> {
        self.knowledge_assets.values()
    }

    /// Get all networks
    pub fn networks(&self) -> &[N] {
        &self.networks
    }

    /// Get the interaction rules
    pub fn interaction_rules(&self) -> &R {
        &self.interaction_rules
    }

    /// Get the current simulation time
    pub fn current_time(&self) -> SimulationTime {
        self.current_time
    }

    /// Update the environment to a new time
    #[cfg(feature = "async")]
    pub async fn update_to_time(&mut self, new_time: SimulationTime) -> Result<Vec<EnvironmentChange>> {
        let mut all_changes = Vec::new();

        // Update physical assets
        for asset in self.physical_assets.values_mut() {
            asset.update_state(new_time)?;
        }

        // Process exogenous processes
        for process in &self.exogenous_processes {
            if process.is_active(new_time) {
                let changes = process.update_environment(new_time).await?;
                all_changes.extend(changes);
            }
        }

        self.current_time = new_time;
        Ok(all_changes)
    }

    #[cfg(not(feature = "async"))]
    pub fn update_to_time(&mut self, new_time: SimulationTime) -> Result<Vec<EnvironmentChange>> {
        let mut all_changes = Vec::new();

        // Update physical assets
        for asset in self.physical_assets.values_mut() {
            asset.update_state(new_time)?;
        }

        // Process exogenous processes
        for process in &self.exogenous_processes {
            if process.is_active(new_time) {
                let changes = process.update_environment(new_time)?;
                all_changes.extend(changes);
            }
        }

        self.current_time = new_time;
        Ok(all_changes)
    }

    /// Get available physical assets at current time
    pub fn available_physical_assets(&self) -> Vec<&P> {
        let current_time = self.current_time;
        self.physical_assets
            .values()
            .filter(|asset| asset.is_available(current_time))
            .collect()
    }

    /// Get knowledge assets accessible to a specific agent
    pub fn accessible_knowledge_assets(&self, agent_id: &AgentId) -> Vec<&K> {
        self.knowledge_assets
            .values()
            .filter(|asset| asset.is_accessible_to(agent_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestPhysicalAsset {
        id: AssetId,
        name: String,
        available: bool,
    }

    impl PhysicalAsset for TestPhysicalAsset {
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
            self.available
        }

        fn update_state(&mut self, _time: SimulationTime) -> Result<()> {
            Ok(())
        }
    }

    #[derive(Debug)]
    struct TestKnowledgeAsset {
        id: AssetId,
        content: String,
        reliability: f64,
    }

    impl KnowledgeAsset for TestKnowledgeAsset {
        fn asset_id(&self) -> &AssetId {
            &self.id
        }

        fn content(&self) -> &str {
            &self.content
        }

        fn reliability(&self) -> f64 {
            self.reliability
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

        fn update_reliability(&mut self, new_reliability: f64) -> Result<()> {
            self.reliability = new_reliability;
            Ok(())
        }
    }

    #[derive(Debug)]
    struct TestNetwork {
        agents: Vec<AgentId>,
    }

    impl Network for TestNetwork {
        fn agents(&self) -> Vec<AgentId> {
            self.agents.clone()
        }

        fn are_connected(&self, _agent1: &AgentId, _agent2: &AgentId) -> bool {
            true
        }

        fn connection_strength(&self, _agent1: &AgentId, _agent2: &AgentId) -> f64 {
            1.0
        }

        fn add_agent(&mut self, agent_id: AgentId) -> Result<()> {
            self.agents.push(agent_id);
            Ok(())
        }

        fn remove_agent(&mut self, agent_id: &AgentId) -> Result<()> {
            self.agents.retain(|id| id != agent_id);
            Ok(())
        }

        fn connect_agents(&mut self, _agent1: AgentId, _agent2: AgentId, _strength: f64) -> Result<()> {
            Ok(())
        }

        fn neighbors(&self, _agent_id: &AgentId) -> Vec<AgentId> {
            self.agents.clone()
        }

        fn network_statistics(&self) -> NetworkStatistics {
            NetworkStatistics {
                agent_count: self.agents.len(),
                connection_count: 0,
                average_degree: 0.0,
                clustering_coefficient: 0.0,
                network_density: 0.0,
            }
        }
    }

    #[derive(Debug)]
    struct TestInteractionRules;

    #[cfg_attr(feature = "async", async_trait)]
    impl RulesOfInteraction for TestInteractionRules {
        type Interaction = String;

        #[cfg(feature = "async")]
        async fn is_interaction_allowed(
            &self,
            _initiator: &AgentId,
            _target: &AgentId,
            _interaction: &Self::Interaction,
            _time: SimulationTime,
        ) -> Result<bool> {
            Ok(true)
        }

        #[cfg(not(feature = "async"))]
        fn is_interaction_allowed(
            &self,
            _initiator: &AgentId,
            _target: &AgentId,
            _interaction: &Self::Interaction,
            _time: SimulationTime,
        ) -> Result<bool> {
            Ok(true)
        }

        #[cfg(feature = "async")]
        async fn process_interaction(
            &self,
            _initiator: &AgentId,
            target: &AgentId,
            _interaction: Self::Interaction,
            _time: SimulationTime,
        ) -> Result<Vec<InteractionEffect>> {
            Ok(vec![InteractionEffect {
                target_agent: target.clone(),
                effect_type: "test".to_string(),
                magnitude: 1.0,
                duration: None,
            }])
        }

        #[cfg(not(feature = "async"))]
        fn process_interaction(
            &self,
            _initiator: &AgentId,
            target: &AgentId,
            _interaction: Self::Interaction,
            _time: SimulationTime,
        ) -> Result<Vec<InteractionEffect>> {
            Ok(vec![InteractionEffect {
                target_agent: target.clone(),
                effect_type: "test".to_string(),
                magnitude: 1.0,
                duration: None,
            }])
        }

        fn interaction_cost(&self, _interaction: &Self::Interaction) -> f64 {
            1.0
        }
    }

    #[derive(Debug)]
    struct TestExogenousProcess;

    #[cfg_attr(feature = "async", async_trait)]
    impl ExogenousProcess for TestExogenousProcess {
        #[cfg(feature = "async")]
        async fn update_environment(&self, _time: SimulationTime) -> Result<Vec<EnvironmentChange>> {
            Ok(vec![])
        }

        #[cfg(not(feature = "async"))]
        fn update_environment(&self, _time: SimulationTime) -> Result<Vec<EnvironmentChange>> {
            Ok(vec![])
        }

        fn is_active(&self, _time: SimulationTime) -> bool {
            true
        }

        fn name(&self) -> &str {
            "test_process"
        }

        fn frequency(&self) -> f64 {
            1.0
        }
    }

    #[test]
    fn test_environment_creation() {
        let rules = TestInteractionRules;
        let env: Environment<TestPhysicalAsset, TestKnowledgeAsset, TestNetwork, TestInteractionRules, TestExogenousProcess> =
            Environment::new(rules);

        assert_eq!(env.current_time(), 0.0);
        assert_eq!(env.physical_assets().count(), 0);
        assert_eq!(env.knowledge_assets().count(), 0);
    }

    #[test]
    fn test_add_physical_asset() {
        let rules = TestInteractionRules;
        let mut env: Environment<TestPhysicalAsset, TestKnowledgeAsset, TestNetwork, TestInteractionRules, TestExogenousProcess> = Environment::new(rules);

        let asset = TestPhysicalAsset {
            id: AssetId::new(),
            name: "Test Asset".to_string(),
            available: true,
        };

        let asset_id = asset.id.clone();
        env.add_physical_asset(asset).unwrap();

        assert!(env.get_physical_asset(&asset_id).is_some());
        assert_eq!(env.physical_assets().count(), 1);
    }
}