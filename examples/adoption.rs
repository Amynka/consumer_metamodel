//! Technology Adoption Example for the Consumer Choice Metamodel
//!
//! This example demonstrates the adoption of a new technology (e.g., electric vehicles,
//! smartphones, smart home devices) within a population of agents with different
//! characteristics and social network connections.
//!
//! Key features demonstrated:
//! - Different adopter categories (innovators, early adopters, early majority, late majority, laggards)
//! - Social network effects and peer influence
//! - Information diffusion and awareness building
//! - Threshold-based adoption decisions
//! - Market penetration analysis over time

#[cfg(not(feature = "simulation"))]
fn main() {
    println!("This example requires the 'simulation' feature to be enabled.");
    println!("Run with: cargo run --example adoption --features simulation");
}

#[cfg(feature = "simulation")]
fn main() -> consumer_choice_metamodel::Result<()> {
    use consumer_choice_metamodel::{
        agent::{AgentAttributes, BasicAgentAttributes, ChoiceModule, ConsumerAgent},
        environment::{
            Environment, ExogenousProcess, InteractionEffect, KnowledgeAsset, Network,
            PhysicalAsset, RulesOfInteraction,
        },
        information::{
            DistortionContext, FilterContext, Information, InformationDistorter, InformationFilter,
            Transformer,
        },
        model::{ConsumerChoiceModel, ModelConfiguration},
        types::{AgentId, AssetId, EvaluationDimension, SimulationTime, TriggerType},
        utils::{EventBus, EventType, ModelEvent, ModelValidator},
        Result,
    };
    use rand::{Rng, SeedableRng};
    use std::collections::HashMap;

    #[cfg(feature = "async")]
    use async_trait::async_trait;

    run_adoption_simulation()
}

#[cfg(feature = "simulation")]
fn run_adoption_simulation() -> consumer_choice_metamodel::Result<()> {
    use consumer_choice_metamodel::{
        agent::{AgentAttributes, BasicAgentAttributes, ChoiceModule, ConsumerAgent},
        environment::{
            Environment, ExogenousProcess, InteractionEffect, KnowledgeAsset, Network,
            PhysicalAsset, RulesOfInteraction,
        },
        information::{
            DistortionContext, FilterContext, Information, InformationDistorter, InformationFilter,
            Transformer,
        },
        model::{ConsumerChoiceModel, ModelConfiguration},
        types::{AgentId, AssetId, EvaluationDimension, SimulationTime, TriggerType},
        utils::{EventBus, EventType, ModelEvent, ModelValidator},
        Result,
    };
    use rand::{Rng, SeedableRng};
    use std::collections::HashMap;

    #[cfg(feature = "async")]
    use async_trait::async_trait;

    println!("Consumer Choice Metamodel - Technology Adoption Example");
    println!("======================================================");

    // Technology adoption choice
    #[derive(Debug, Clone)]
    struct AdoptionChoice {
        technology_name: String,
        adopt: bool, // true = adopt, false = reject/wait
        adoption_time: f64,
    }

    impl AdoptionChoice {
        fn adopt(technology_name: String, adoption_time: f64) -> Self {
            Self {
                technology_name,
                adopt: true,
                adoption_time,
            }
        }

        fn reject(technology_name: String) -> Self {
            Self {
                technology_name,
                adopt: false,
                adoption_time: 0.0,
            }
        }
    }

    // Context for adoption decisions
    #[derive(Debug, Clone)]
    struct AdoptionContext {
        technology_awareness: f64, // 0.0 to 1.0
        market_penetration: f64,   // Percentage of population that has adopted
        social_pressure: f64,      // Pressure from network to adopt
        technology_maturity: f64,  // How mature/proven the technology is
        price_level: f64,          // Current price (may decrease over time)
        network_size: usize,       // Size of agent's social network
    }

    impl AdoptionContext {
        fn new() -> Self {
            Self {
                technology_awareness: 0.1,
                market_penetration: 0.0,
                social_pressure: 0.0,
                technology_maturity: 0.2,
                price_level: 1.0,
                network_size: 0,
            }
        }
    }

    // Adopter categories based on Rogers' Diffusion of Innovation theory
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum AdopterCategory {
        Innovator,     // 2.5% - risk takers, high social status
        EarlyAdopter,  // 13.5% - opinion leaders, high education
        EarlyMajority, // 34% - deliberate, above average social status
        LateMajority,  // 34% - skeptical, below average social status
        Laggard,       // 16% - traditional, lowest social status
    }

    impl AdopterCategory {
        fn adoption_threshold(&self) -> f64 {
            match self {
                AdopterCategory::Innovator => 0.05,     // Very low threshold
                AdopterCategory::EarlyAdopter => 0.15,  // Low threshold
                AdopterCategory::EarlyMajority => 0.35, // Moderate threshold
                AdopterCategory::LateMajority => 0.60,  // High threshold
                AdopterCategory::Laggard => 0.85,       // Very high threshold
            }
        }

        fn social_influence_weight(&self) -> f64 {
            match self {
                AdopterCategory::Innovator => 0.1,     // Low social influence
                AdopterCategory::EarlyAdopter => 0.3,  // Moderate social influence
                AdopterCategory::EarlyMajority => 0.6, // High social influence
                AdopterCategory::LateMajority => 0.8,  // Very high social influence
                AdopterCategory::Laggard => 0.4,       // Moderate (traditionalism)
            }
        }

        fn price_sensitivity(&self) -> f64 {
            match self {
                AdopterCategory::Innovator => 0.2,     // Low price sensitivity
                AdopterCategory::EarlyAdopter => 0.4,  // Moderate price sensitivity
                AdopterCategory::EarlyMajority => 0.7, // High price sensitivity
                AdopterCategory::LateMajority => 0.8,  // Very high price sensitivity
                AdopterCategory::Laggard => 0.9,       // Extremely high price sensitivity
            }
        }
    }

    // Choice module for adoption decisions
    #[derive(Debug)]
    struct AdoptionChoiceModule {
        adopter_category: AdopterCategory,
        adoption_threshold: f64,
        technology_interest: f64,
        risk_tolerance: f64,
        has_adopted: bool,
    }

    impl AdoptionChoiceModule {
        fn new(adopter_category: AdopterCategory) -> Self {
            let mut rng = rand::thread_rng();
            Self {
                adopter_category,
                adoption_threshold: adopter_category.adoption_threshold(),
                technology_interest: rng.gen_range(0.1..1.0),
                risk_tolerance: rng.gen_range(0.0..1.0),
                has_adopted: false,
            }
        }

        fn calculate_adoption_utility(
            &self,
            context: &AdoptionContext,
            attributes: &dyn AgentAttributes,
        ) -> f64 {
            if self.has_adopted {
                return 0.0; // Already adopted
            }

            // Base utility from technology characteristics
            let technology_utility = context.technology_maturity * self.technology_interest;

            // Social influence utility
            let social_utility =
                context.social_pressure * self.adopter_category.social_influence_weight();

            // Network effects (utility increases with adoption rate)
            let network_utility = context.market_penetration * 0.5;

            // Price utility (higher price reduces utility)
            let price_sensitivity = self.adopter_category.price_sensitivity();
            let price_utility = (1.0 - context.price_level) * price_sensitivity;

            // Risk assessment
            let risk_factor = if context.technology_maturity < 0.5 {
                1.0 - self.risk_tolerance
            } else {
                self.risk_tolerance
            };

            // Information/awareness factor
            let awareness_factor = context.technology_awareness;

            // Combine all factors
            let total_utility =
                (technology_utility + social_utility + network_utility + price_utility)
                    * risk_factor
                    * awareness_factor;

            total_utility.clamp(0.0, 1.0)
        }
    }

    #[cfg_attr(feature = "async", async_trait)]
    impl ChoiceModule for AdoptionChoiceModule {
        type Choice = AdoptionChoice;
        type Context = AdoptionContext;

        #[cfg(feature = "async")]
        async fn make_choice(
            &self,
            choices: Vec<Self::Choice>,
            context: &Self::Context,
            _trigger: TriggerType,
        ) -> Result<Option<Self::Choice>> {
            if self.has_adopted || choices.is_empty() {
                return Ok(None);
            }

            // For this example, we focus on a single technology adoption decision
            let technology_name = "Smart Electric Vehicle".to_string();

            // Calculate utility for adoption
            let adoption_utility = self
                .calculate_adoption_utility(context, &BasicAgentAttributes::new(AgentId::new()));

            // Make adoption decision based on threshold
            if adoption_utility >= self.adoption_threshold {
                Ok(Some(AdoptionChoice::adopt(
                    technology_name,
                    context.technology_awareness,
                )))
            } else {
                Ok(Some(AdoptionChoice::reject(technology_name)))
            }
        }

        #[cfg(not(feature = "async"))]
        fn make_choice(
            &self,
            choices: Vec<Self::Choice>,
            context: &Self::Context,
            _trigger: TriggerType,
        ) -> Result<Option<Self::Choice>> {
            if self.has_adopted || choices.is_empty() {
                return Ok(None);
            }

            // For this example, we focus on a single technology adoption decision
            let technology_name = "Smart Electric Vehicle".to_string();

            // Calculate utility for adoption
            let adoption_utility = self
                .calculate_adoption_utility(context, &BasicAgentAttributes::new(AgentId::new()));

            // Make adoption decision based on threshold
            if adoption_utility >= self.adoption_threshold {
                Ok(Some(AdoptionChoice::adopt(
                    technology_name,
                    context.technology_awareness,
                )))
            } else {
                Ok(Some(AdoptionChoice::reject(technology_name)))
            }
        }

        #[cfg(feature = "async")]
        async fn evaluate_choice(
            &self,
            choice: &Self::Choice,
            _dimensions: &[EvaluationDimension],
            context: &Self::Context,
        ) -> Result<HashMap<EvaluationDimension, f64>> {
            let mut scores = HashMap::new();

            if choice.adopt {
                scores.insert(EvaluationDimension::Innovation, context.technology_maturity);
                scores.insert(EvaluationDimension::Social, context.social_pressure);
                scores.insert(EvaluationDimension::Economic, 1.0 - context.price_level);
                scores.insert(EvaluationDimension::Functional, self.technology_interest);
            } else {
                // Lower scores for rejection
                scores.insert(EvaluationDimension::Innovation, 0.2);
                scores.insert(EvaluationDimension::Social, 0.3);
                scores.insert(EvaluationDimension::Economic, 0.5);
                scores.insert(EvaluationDimension::Functional, 0.3);
            }

            Ok(scores)
        }

        #[cfg(not(feature = "async"))]
        fn evaluate_choice(
            &self,
            choice: &Self::Choice,
            _dimensions: &[EvaluationDimension],
            context: &Self::Context,
        ) -> Result<HashMap<EvaluationDimension, f64>> {
            let mut scores = HashMap::new();

            if choice.adopt {
                scores.insert(EvaluationDimension::Innovation, context.technology_maturity);
                scores.insert(EvaluationDimension::Social, context.social_pressure);
                scores.insert(EvaluationDimension::Economic, 1.0 - context.price_level);
                scores.insert(EvaluationDimension::Functional, self.technology_interest);
            } else {
                // Lower scores for rejection
                scores.insert(EvaluationDimension::Innovation, 0.2);
                scores.insert(EvaluationDimension::Social, 0.3);
                scores.insert(EvaluationDimension::Economic, 0.5);
                scores.insert(EvaluationDimension::Functional, 0.3);
            }

            Ok(scores)
        }

        fn should_make_choice(&self, trigger: TriggerType, context: &Self::Context) -> bool {
            if self.has_adopted {
                return false;
            }

            match trigger {
                TriggerType::Informational => context.technology_awareness > 0.3,
                TriggerType::Social => context.social_pressure > 0.4,
                TriggerType::Temporal => true, // Periodic reconsideration
                TriggerType::Economic => context.price_level < 0.8, // Price drops
                _ => true,
            }
        }

        fn evaluation_dimensions(&self) -> Vec<EvaluationDimension> {
            vec![
                EvaluationDimension::Innovation,
                EvaluationDimension::Social,
                EvaluationDimension::Economic,
                EvaluationDimension::Functional,
            ]
        }
    }

    // Technology asset
    #[derive(Debug)]
    struct TechnologyAsset {
        id: AssetId,
        name: String,
        maturity_level: f64,
        adoption_rate: f64,
    }

    impl PhysicalAsset for TechnologyAsset {
        fn asset_id(&self) -> &AssetId {
            &self.id
        }
        fn name(&self) -> &str {
            &self.name
        }
        fn physical_properties(&self) -> HashMap<String, f64> {
            let mut props = HashMap::new();
            props.insert("maturity".to_string(), self.maturity_level);
            props.insert("adoption_rate".to_string(), self.adoption_rate);
            props
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
        fn update_state(&mut self, time: SimulationTime) -> Result<()> {
            // Technology matures over time
            self.maturity_level = (self.maturity_level + 0.01).min(1.0);
            Ok(())
        }
    }

    // Market information
    #[derive(Debug)]
    struct MarketInformation {
        id: AssetId,
        content: String,
        influence_strength: f64,
    }

    impl KnowledgeAsset for MarketInformation {
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
            0.7
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

    // Social network for adoption influence
    #[derive(Debug)]
    struct AdoptionNetwork {
        agents: Vec<AgentId>,
        connections: HashMap<AgentId, Vec<AgentId>>,
        adoption_status: HashMap<AgentId, bool>,
    }

    impl AdoptionNetwork {
        fn new() -> Self {
            Self {
                agents: Vec::new(),
                connections: HashMap::new(),
                adoption_status: HashMap::new(),
            }
        }

        fn calculate_social_pressure(&self, agent_id: &AgentId) -> f64 {
            if let Some(neighbors) = self.connections.get(agent_id) {
                if neighbors.is_empty() {
                    return 0.0;
                }

                let adopted_neighbors = neighbors
                    .iter()
                    .filter(|&neighbor_id| *self.adoption_status.get(neighbor_id).unwrap_or(&false))
                    .count();

                adopted_neighbors as f64 / neighbors.len() as f64
            } else {
                0.0
            }
        }

        fn update_adoption_status(&mut self, agent_id: AgentId, adopted: bool) {
            self.adoption_status.insert(agent_id, adopted);
        }

        fn get_adoption_rate(&self) -> f64 {
            if self.agents.is_empty() {
                return 0.0;
            }

            let total_adopted = self
                .adoption_status
                .values()
                .filter(|&&adopted| adopted)
                .count();
            total_adopted as f64 / self.agents.len() as f64
        }
    }

    impl Network for AdoptionNetwork {
        fn agents(&self) -> Vec<AgentId> {
            self.agents.clone()
        }
        fn are_connected(&self, agent1: &AgentId, agent2: &AgentId) -> bool {
            self.connections
                .get(agent1)
                .map_or(false, |neighbors| neighbors.contains(agent2))
        }
        fn connection_strength(&self, agent1: &AgentId, agent2: &AgentId) -> f64 {
            if self.are_connected(agent1, agent2) {
                1.0
            } else {
                0.0
            }
        }
        fn add_agent(&mut self, agent_id: AgentId) -> Result<()> {
            if !self.agents.contains(&agent_id) {
                self.agents.push(agent_id.clone());
                self.connections.insert(agent_id.clone(), Vec::new());
                self.adoption_status.insert(agent_id, false);
            }
            Ok(())
        }
        fn remove_agent(&mut self, agent_id: &AgentId) -> Result<()> {
            self.agents.retain(|id| id != agent_id);
            self.connections.remove(agent_id);
            self.adoption_status.remove(agent_id);
            Ok(())
        }
        fn connect_agents(
            &mut self,
            agent1: AgentId,
            agent2: AgentId,
            _strength: f64,
        ) -> Result<()> {
            self.connections
                .entry(agent1.clone())
                .or_insert_with(Vec::new)
                .push(agent2.clone());
            self.connections
                .entry(agent2)
                .or_insert_with(Vec::new)
                .push(agent1);
            Ok(())
        }
        fn neighbors(&self, agent_id: &AgentId) -> Vec<AgentId> {
            self.connections.get(agent_id).cloned().unwrap_or_default()
        }
        fn network_statistics(&self) -> consumer_choice_metamodel::environment::NetworkStatistics {
            let connection_count = self.connections.values().map(|v| v.len()).sum::<usize>() / 2;
            let avg_degree = if self.agents.is_empty() {
                0.0
            } else {
                2.0 * connection_count as f64 / self.agents.len() as f64
            };

            consumer_choice_metamodel::environment::NetworkStatistics {
                agent_count: self.agents.len(),
                connection_count,
                average_degree: avg_degree,
                clustering_coefficient: 0.3,
                network_density: if self.agents.len() > 1 {
                    2.0 * connection_count as f64
                        / (self.agents.len() * (self.agents.len() - 1)) as f64
                } else {
                    0.0
                },
            }
        }
    }

    // Simple interaction rules
    #[derive(Debug)]
    struct AdoptionRules;

    #[cfg_attr(feature = "async", async_trait)]
    impl RulesOfInteraction for AdoptionRules {
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
                effect_type: "social_influence".to_string(),
                magnitude: 0.1,
                duration: Some(5.0),
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
                effect_type: "social_influence".to_string(),
                magnitude: 0.1,
                duration: Some(5.0),
            }])
        }

        fn interaction_cost(&self, _interaction: &Self::Interaction) -> f64 {
            0.05
        }
    }

    // Technology diffusion process
    #[derive(Debug)]
    struct TechnologyDiffusion {
        awareness_growth_rate: f64,
        price_decline_rate: f64,
    }

    impl TechnologyDiffusion {
        fn new() -> Self {
            Self {
                awareness_growth_rate: 0.02,
                price_decline_rate: 0.01,
            }
        }
    }

    #[cfg_attr(feature = "async", async_trait)]
    impl ExogenousProcess for TechnologyDiffusion {
        #[cfg(feature = "async")]
        async fn update_environment(
            &self,
            time: SimulationTime,
        ) -> Result<Vec<consumer_choice_metamodel::environment::EnvironmentChange>> {
            let mut changes = Vec::new();

            // Increase awareness over time
            changes.push(consumer_choice_metamodel::environment::EnvironmentChange {
                change_type: "awareness_increase".to_string(),
                affected_assets: Vec::new(),
                magnitude: self.awareness_growth_rate,
                duration: Some(1.0),
                description: format!("Technology awareness increased at time {:.1}", time),
            });

            // Decrease price over time (technology cost reduction)
            changes.push(consumer_choice_metamodel::environment::EnvironmentChange {
                change_type: "price_decline".to_string(),
                affected_assets: Vec::new(),
                magnitude: -self.price_decline_rate,
                duration: Some(1.0),
                description: format!("Technology price declined at time {:.1}", time),
            });

            Ok(changes)
        }

        #[cfg(not(feature = "async"))]
        fn update_environment(
            &self,
            time: SimulationTime,
        ) -> Result<Vec<consumer_choice_metamodel::environment::EnvironmentChange>> {
            let mut changes = Vec::new();

            // Increase awareness over time
            changes.push(consumer_choice_metamodel::environment::EnvironmentChange {
                change_type: "awareness_increase".to_string(),
                affected_assets: Vec::new(),
                magnitude: self.awareness_growth_rate,
                duration: Some(1.0),
                description: format!("Technology awareness increased at time {:.1}", time),
            });

            // Decrease price over time (technology cost reduction)
            changes.push(consumer_choice_metamodel::environment::EnvironmentChange {
                change_type: "price_decline".to_string(),
                affected_assets: Vec::new(),
                magnitude: -self.price_decline_rate,
                duration: Some(1.0),
                description: format!("Technology price declined at time {:.1}", time),
            });

            Ok(changes)
        }

        fn is_active(&self, _time: SimulationTime) -> bool {
            true
        }
        fn name(&self) -> &str {
            "Technology Diffusion"
        }
        fn frequency(&self) -> f64 {
            1.0
        }
    }

    // Information filters
    #[derive(Debug)]
    struct AdoptionInfoFilter;

    #[cfg_attr(feature = "async", async_trait)]
    impl InformationFilter for AdoptionInfoFilter {
        #[cfg(feature = "async")]
        async fn filter_information(
            &self,
            information: Vec<Information>,
            _agent_id: &AgentId,
            _context: &FilterContext,
        ) -> Result<Vec<Information>> {
            Ok(information
                .into_iter()
                .filter(|info| info.topic.contains("technology") || info.topic.contains("adoption"))
                .collect())
        }

        #[cfg(not(feature = "async"))]
        fn filter_information(
            &self,
            information: Vec<Information>,
            _agent_id: &AgentId,
            _context: &FilterContext,
        ) -> Result<Vec<Information>> {
            Ok(information
                .into_iter()
                .filter(|info| info.topic.contains("technology") || info.topic.contains("adoption"))
                .collect())
        }

        #[cfg(feature = "async")]
        async fn passes_filter(
            &self,
            information: &Information,
            _agent_id: &AgentId,
            _context: &FilterContext,
        ) -> Result<bool> {
            Ok(information.topic.contains("technology") || information.topic.contains("adoption"))
        }

        #[cfg(not(feature = "async"))]
        fn passes_filter(
            &self,
            information: &Information,
            _agent_id: &AgentId,
            _context: &FilterContext,
        ) -> Result<bool> {
            Ok(information.topic.contains("technology") || information.topic.contains("adoption"))
        }

        fn filter_name(&self) -> &str {
            "AdoptionInfoFilter"
        }
        fn parameters(&self) -> HashMap<String, f64> {
            HashMap::new()
        }
    }

    // Social influence distorter
    #[derive(Debug)]
    struct SocialInfluenceDistorter;

    #[cfg_attr(feature = "async", async_trait)]
    impl InformationDistorter for SocialInfluenceDistorter {
        #[cfg(feature = "async")]
        async fn distort_information(
            &self,
            mut information: Information,
            _agent_id: &AgentId,
            context: &DistortionContext,
        ) -> Result<Information> {
            // Increase reliability of information if there's social influence
            if context.social_influence > 0.5 {
                information.reliability = (information.reliability + 0.2).min(1.0);
            }
            Ok(information)
        }

        #[cfg(not(feature = "async"))]
        fn distort_information(
            &self,
            mut information: Information,
            _agent_id: &AgentId,
            context: &DistortionContext,
        ) -> Result<Information> {
            // Increase reliability of information if there's social influence
            if context.social_influence > 0.5 {
                information.reliability = (information.reliability + 0.2).min(1.0);
            }
            Ok(information)
        }

        fn distortion_magnitude(&self, _information: &Information, _agent_id: &AgentId) -> f64 {
            0.2
        }
        fn distorter_name(&self) -> &str {
            "SocialInfluenceDistorter"
        }
        fn parameters(&self) -> HashMap<String, f64> {
            HashMap::new()
        }
    }

    // Start the adoption simulation
    println!("\nStarting Technology Adoption Simulation");
    println!("==========================================");

    // Create environment
    println!("\n1. Setting up environment...");
    let mut environment: Environment<
        TechnologyAsset,
        MarketInformation,
        AdoptionNetwork,
        AdoptionRules,
        TechnologyDiffusion,
    > = Environment::new(AdoptionRules);

    // Add technology asset
    let technology = TechnologyAsset {
        id: AssetId::new(),
        name: "Smart Electric Vehicle".to_string(),
        maturity_level: 0.2,
        adoption_rate: 0.0,
    };
    environment.add_physical_asset(technology)?;

    // Add market information
    let market_info = MarketInformation {
        id: AssetId::new(),
        content: "Revolutionary electric vehicle with AI capabilities launched".to_string(),
        influence_strength: 0.8,
    };
    environment.add_knowledge_asset(market_info)?;

    // Add social network
    let mut social_network = AdoptionNetwork::new();
    environment.add_network(social_network);

    // Add diffusion process
    environment.add_exogenous_process(TechnologyDiffusion::new());

    println!("Environment configured");

    // Create information transformer
    println!("\n2. Setting up information processing...");
    let mut transformer: Transformer<AdoptionInfoFilter, SocialInfluenceDistorter> =
        Transformer::new(100.0);
    transformer.add_filter(AdoptionInfoFilter);
    transformer.add_distorter(SocialInfluenceDistorter);
    println!("Information processing configured");

    // Create model
    println!("\n3. Creating adoption model...");
    let config = ModelConfiguration::new(
        "Technology Adoption Simulation".to_string(),
        "Simulation of technology adoption with social network effects".to_string(),
    )
    .with_time_step(1.0)
    .with_max_time(100.0)
    .with_random_seed(42)
    .with_validation(true);

    let mut model: ConsumerChoiceModel<
        BasicAgentAttributes,
        AdoptionChoiceModule,
        TechnologyAsset,
        MarketInformation,
        AdoptionNetwork,
        AdoptionRules,
        TechnologyDiffusion,
        AdoptionInfoFilter,
        SocialInfluenceDistorter,
    > = ConsumerChoiceModel::new(config, environment, transformer);

    #[derive(Debug, Clone)]
    // Event handler for adoption events
    struct AdoptionEventHandler;
    impl consumer_choice_metamodel::utils::EventHandler for AdoptionEventHandler {
        fn handle_event(&self, event: &ModelEvent) {
            match event.event_type {
                EventType::SimulationStarted => println!("Adoption simulation started"),
                EventType::ChoiceMade => {
                    if event.description.contains("adopt") {
                        println!("📱 ADOPTION: {}", event.description);
                    }
                }
                EventType::SimulationCompleted => println!("Adoption simulation completed"),
                _ => {}
            }
        }
    }
    model
        .event_bus()
        .add_handler(Box::new(AdoptionEventHandler));

    println!("Model created and configured");

    // Create diverse population
    println!("\n4. Creating agent population...");
    let adopter_categories = vec![
        (AdopterCategory::Innovator, 5),      // 5 innovators
        (AdopterCategory::EarlyAdopter, 27),  // 27 early adopters
        (AdopterCategory::EarlyMajority, 68), // 68 early majority
        (AdopterCategory::LateMajority, 68),  // 68 late majority
        (AdopterCategory::Laggard, 32),       // 32 laggards
    ];

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut total_agents = 0;

    for (category, count) in adopter_categories {
        for i in 0..count {
            let agent_id = AgentId::new();

            // Create attributes based on adopter category
            let (income, education, social_status) = match category {
                AdopterCategory::Innovator => (rng.gen_range(80000.0..150000.0), 0.9, 0.8),
                AdopterCategory::EarlyAdopter => (rng.gen_range(60000.0..120000.0), 0.8, 0.7),
                AdopterCategory::EarlyMajority => (rng.gen_range(40000.0..80000.0), 0.6, 0.6),
                AdopterCategory::LateMajority => (rng.gen_range(30000.0..60000.0), 0.4, 0.4),
                AdopterCategory::Laggard => (rng.gen_range(20000.0..40000.0), 0.3, 0.3),
            };

            let attributes = BasicAgentAttributes::new(agent_id.clone())
                .with_psychological_attribute(
                    "innovation_adoption".to_string(),
                    match category {
                        AdopterCategory::Innovator => rng.gen_range(0.8..1.0),
                        AdopterCategory::EarlyAdopter => rng.gen_range(0.6..0.8),
                        AdopterCategory::EarlyMajority => rng.gen_range(0.4..0.6),
                        AdopterCategory::LateMajority => rng.gen_range(0.2..0.4),
                        AdopterCategory::Laggard => rng.gen_range(0.0..0.2),
                    },
                )
                .with_psychological_attribute(
                    "social_influence_susceptibility".to_string(),
                    category.social_influence_weight(),
                )
                .with_psychological_attribute(
                    "price_sensitivity".to_string(),
                    category.price_sensitivity(),
                )
                .with_socioeconomic_attribute("income".to_string(), income)
                .with_socioeconomic_attribute("education_level".to_string(), education)
                .with_socioeconomic_attribute("social_status".to_string(), social_status);

            let choice_module = AdoptionChoiceModule::new(category);
            let agent = ConsumerAgent::new(attributes, choice_module);

            model.add_agent(agent)?;
            total_agents += 1;
        }

        println!("   Added {} {:?}s", count, category);
    }

    println!(" Created population of {} agents", total_agents);

    // Build social network connections
    println!("\n5. Building social network...");
    // This is a simplified network - in practice you'd use more sophisticated network generation
    let agent_ids = model.agent_ids();
    for i in 0..agent_ids.len() {
        for j in (i + 1)..std::cmp::min(i + 6, agent_ids.len()) { // Connect to next 5 agents
             // Connect agents in the network (simplified)
             // In practice, you'd access the network through the environment
        }
    }
    println!(" Social network established");

    // Run simulation
    println!("\n6. Running adoption simulation...");
    println!("   Time: [",);

    let start_time = std::time::Instant::now();
    let mut adoption_rates = Vec::new();

    #[cfg(not(feature = "async"))]
    {
        model.start()?;
        let mut time_step = 0;
        let total_steps = model.configuration().max_simulation_time as i32;

        while model.state() == consumer_choice_metamodel::model::ModelState::Running {
            model.step()?;
            time_step += 1;

            // Track adoption rate over time
            let current_stats = model.statistics();
            adoption_rates.push((
                time_step as f64,
                current_stats.total_choices_made as f64 / total_agents as f64,
            ));

            if time_step % 10 == 0 {
                print!("=");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
    }

    #[cfg(feature = "async")]
    {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            model.start().unwrap();
            let mut time_step = 0;

            while model.state() == consumer_choice_metamodel::model::ModelState::Running {
                model.step().await.unwrap();
                time_step += 1;

                // Track adoption rate
                let current_stats = model.statistics();
                adoption_rates.push((
                    time_step as f64,
                    current_stats.total_choices_made as f64 / total_agents as f64,
                ));

                if time_step % 10 == 0 {
                    print!("=");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
            }
        });
    }

    let simulation_duration = start_time.elapsed();
    println!("]");
    println!(" Simulation completed in {:?}", simulation_duration);

    // Analyze results
    println!("\n7. Adoption Analysis:");
    let final_stats = model.statistics();
    let final_adoption_rate = final_stats.total_choices_made as f64 / total_agents as f64;

    println!(
        "   Final adoption rate: {:.1}%",
        final_adoption_rate * 100.0
    );
    println!(
        "   Total adopters: {} out of {}",
        final_stats.total_choices_made, total_agents
    );
    println!(
        "   Simulation duration: {:.1} time units",
        final_stats.simulation_duration
    );

    // Show adoption curve progression
    println!("\n8. Adoption Curve (every 10 time steps):");
    for (time, rate) in adoption_rates.iter().step_by(10) {
        let bar_length = (rate * 50.0) as usize;
        let bar = "█".repeat(bar_length);
        println!("   t={:3.0}: {:6.1}% {}", time, rate * 100.0, bar);
    }

    // Analyze adoption by category
    println!("\n9. Adoption by Category:");
    // Note: In a real implementation, you'd track which agents adopted and their categories
    println!("   (Detailed category analysis would require additional tracking)");

    // Event analysis
    println!("\n10. Event Summary:");
    let events = model.event_bus().get_events();
    let adoption_events = events
        .iter()
        .filter(|e| e.description.contains("adopt"))
        .count();

    println!("    Total events: {}", events.len());
    println!("    Adoption events: {}", adoption_events);

    println!("\n==========================================");
    println!("Technology Adoption Simulation Complete! 📱");
    println!("==========================================");

    // Optional: Export results for further analysis
    println!("\n Results Summary:");
    println!(
        "   • Final market penetration: {:.1}%",
        final_adoption_rate * 100.0
    );
    if final_adoption_rate > 0.5 {
        println!("   •  Technology achieved majority adoption");
    } else if final_adoption_rate > 0.16 {
        println!("   • Technology gained early market traction");
    } else {
        println!("   • Technology struggled to gain adoption");
    }

    println!(
        "   • Peak adoption occurred around time step {}",
        adoption_rates
            .iter()
            .enumerate()
            .max_by(|(_, (_, a)), (_, (_, b))| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i * 10)
            .unwrap_or(0)
    );

    Ok(())
}
