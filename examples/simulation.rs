//! Complete simulation example for the Consumer Choice Metamodel
//!
//! This example demonstrates:
//! - Creating multiple agents with different characteristics
//! - Setting up a complex environment with assets and networks
//! - Running a multi-step simulation
//! - Processing information and environmental changes
//! - Analyzing simulation results

#[cfg(not(feature = "simulation"))]
fn main() {
    println!("This example requires the 'simulation' feature to be enabled.");
    println!("Run with: cargo run --example simulation --features simulation");
}

#[cfg(feature = "simulation")]
fn main() -> consumer_choice_metamodel::Result<()> {
    use consumer_choice_metamodel::{
        agent::{AgentAttributes, BasicAgentAttributes, ChoiceModule, ConsumerAgent},
        environment::{Environment, PhysicalAsset, KnowledgeAsset, Network, RulesOfInteraction, ExogenousProcess},
        information::{Information, InformationFilter, InformationDistorter, Transformer, FilterContext, DistortionContext},
        model::{ConsumerChoiceModel, ModelConfiguration},
        types::{AgentId, AssetId, EvaluationDimension, SimulationTime, TriggerType},
        utils::{EventBus, ModelValidator, ModelEvent, EventType},
        Result,
    };
    use rand::{Rng, SeedableRng};
    use std::collections::HashMap;

    #[cfg(feature = "async")]
    use async_trait::async_trait;

    run_simulation()
}

#[cfg(feature = "simulation")]
fn run_simulation() -> consumer_choice_metamodel::Result<()> {
    use consumer_choice_metamodel::{
        agent::{AgentAttributes, BasicAgentAttributes, ChoiceModule, ConsumerAgent},
        environment::{Environment, PhysicalAsset, KnowledgeAsset, Network, RulesOfInteraction, ExogenousProcess},
        information::{Information, InformationFilter, InformationDistorter, Transformer, FilterContext, DistortionContext},
        model::{ConsumerChoiceModel, ModelConfiguration},
        types::{AgentId, AssetId, EvaluationDimension, SimulationTime, TriggerType},
        utils::{EventBus, ModelValidator, ModelEvent, EventType},
        Result,
    };
    use rand::{Rng, SeedableRng};
    use std::collections::HashMap;

    #[cfg(feature = "async")]
    use async_trait::async_trait;

    println!("Consumer Choice Metamodel - Simulation Example");
    println!("==============================================");

    // Vehicle choice for the simulation
    #[derive(Debug, Clone)]
    struct VehicleChoice {
        name: String,
        brand: String,
        price: f64,
        fuel_efficiency: f64,
        safety_rating: f64,
        environmental_score: f64,
        luxury_level: f64,
    }

    impl VehicleChoice {
        fn new(name: String, brand: String, price: f64, fuel_efficiency: f64, safety_rating: f64, environmental_score: f64, luxury_level: f64) -> Self {
            Self {
                name,
                brand,
                price,
                fuel_efficiency,
                safety_rating,
                environmental_score,
                luxury_level,
            }
        }
    }

    // Market context for vehicle purchases
    #[derive(Debug, Clone)]
    struct MarketContext {
        economic_conditions: f64, // 0.0 = recession, 1.0 = boom
        fuel_prices: f64,
        environmental_awareness: f64,
        available_budget: f64,
        financing_available: bool,
    }

    impl MarketContext {
        fn new() -> Self {
            Self {
                economic_conditions: 0.7,
                fuel_prices: 3.5,
                environmental_awareness: 0.6,
                available_budget: 40000.0,
                financing_available: true,
            }
        }
    }

    // Sophisticated choice module for vehicle selection
    #[derive(Debug)]
    struct VehicleChoiceModule {
        risk_tolerance: f64,
        brand_loyalty: HashMap<String, f64>,
        decision_style: DecisionStyle,
    }

    #[derive(Debug)]
    enum DecisionStyle {
        Analytical,    // Considers all factors carefully
        Emotional,     // Focuses on personal preferences
        Social,        // Influenced by others' choices
        Economic,      // Primarily price-driven
    }

    impl VehicleChoiceModule {
        fn new(risk_tolerance: f64, decision_style: DecisionStyle) -> Self {
            Self {
                risk_tolerance,
                brand_loyalty: HashMap::new(),
                decision_style,
            }
        }

        fn calculate_utility(&self, choice: &VehicleChoice, attributes: &dyn AgentAttributes, context: &MarketContext) -> f64 {
            let price_sensitivity = attributes.get_psychological_attribute("price_sensitivity").unwrap_or(0.5);
            let environmental_concern = attributes.get_psychological_attribute("environmental_concern").unwrap_or(0.5);
            let safety_concern = attributes.get_psychological_attribute("safety_concern").unwrap_or(0.5);
            let status_seeking = attributes.get_psychological_attribute("status_seeking").unwrap_or(0.5);

            let income = attributes.get_socioeconomic_attribute("income").unwrap_or(50000.0);

            // Economic utility
            let affordability = (income * 0.3 - choice.price).max(0.0) / (income * 0.3);
            let economic_utility = (1.0 - price_sensitivity) + price_sensitivity * affordability;

            // Environmental utility
            let environmental_utility = choice.environmental_score * environmental_concern * context.environmental_awareness;

            // Safety utility
            let safety_utility = choice.safety_rating * safety_concern;

            // Status utility
            let status_utility = choice.luxury_level * status_seeking;

            // Fuel efficiency utility (higher when fuel prices are high)
            let fuel_utility = choice.fuel_efficiency * (context.fuel_prices / 5.0);

            // Brand loyalty bonus
            let brand_bonus = self.brand_loyalty.get(&choice.brand).unwrap_or(&0.0);

            // Weight utilities based on decision style
            let utility = match self.decision_style {
                DecisionStyle::Analytical => {
                    0.25 * economic_utility + 0.20 * environmental_utility +
                    0.25 * safety_utility + 0.15 * fuel_utility + 0.15 * status_utility + brand_bonus
                }
                DecisionStyle::Emotional => {
                    0.15 * economic_utility + 0.10 * environmental_utility +
                    0.20 * safety_utility + 0.10 * fuel_utility + 0.45 * status_utility + brand_bonus
                }
                DecisionStyle::Social => {
                    0.20 * economic_utility + 0.15 * environmental_utility +
                    0.15 * safety_utility + 0.15 * fuel_utility + 0.35 * status_utility + brand_bonus * 2.0
                }
                DecisionStyle::Economic => {
                    0.50 * economic_utility + 0.10 * environmental_utility +
                    0.15 * safety_utility + 0.25 * fuel_utility + 0.0 * status_utility + brand_bonus * 0.5
                }
            };

            utility.clamp(0.0, 1.0)
        }
    }

    #[cfg_attr(feature = "async", async_trait)]
    impl ChoiceModule for VehicleChoiceModule {
        type Choice = VehicleChoice;
        type Context = MarketContext;

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

            // This would be implemented with proper attribute access
            // For now, return the first choice as a placeholder
            Ok(choices.into_iter().next())
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

            // Filter affordable choices
            let affordable_choices: Vec<_> = choices.into_iter()
                .filter(|choice| choice.price <= context.available_budget || context.financing_available)
                .collect();

            if affordable_choices.is_empty() {
                return Ok(None);
            }

            // For this example, select randomly among affordable choices
            // In a real implementation, you would calculate utilities
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..affordable_choices.len());
            Ok(Some(affordable_choices[index].clone()))
        }

        #[cfg(feature = "async")]
        async fn evaluate_choice(
            &self,
            choice: &Self::Choice,
            dimensions: &[EvaluationDimension],
            context: &Self::Context,
        ) -> Result<HashMap<EvaluationDimension, f64>> {
            let mut scores = HashMap::new();

            for dimension in dimensions {
                let score = match dimension {
                    EvaluationDimension::Economic => 1.0 - (choice.price / 100000.0).min(1.0),
                    EvaluationDimension::Environmental => choice.environmental_score,
                    EvaluationDimension::Safety => choice.safety_rating,
                    EvaluationDimension::Functional => choice.fuel_efficiency / 50.0,
                    EvaluationDimension::Aesthetic => choice.luxury_level,
                    _ => 0.5,
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
            context: &Self::Context,
        ) -> Result<HashMap<EvaluationDimension, f64>> {
            let mut scores = HashMap::new();

            for dimension in dimensions {
                let score = match dimension {
                    EvaluationDimension::Economic => 1.0 - (choice.price / 100000.0).min(1.0),
                    EvaluationDimension::Environmental => choice.environmental_score,
                    EvaluationDimension::Safety => choice.safety_rating,
                    EvaluationDimension::Functional => choice.fuel_efficiency / 50.0,
                    EvaluationDimension::Aesthetic => choice.luxury_level,
                    _ => 0.5,
                };
                scores.insert(dimension.clone(), score.clamp(0.0, 1.0));
            }

            Ok(scores)
        }

        fn should_make_choice(&self, trigger: TriggerType, context: &Self::Context) -> bool {
            match trigger {
                TriggerType::Economic => context.economic_conditions > 0.3,
                TriggerType::Environmental => context.environmental_awareness > 0.4,
                TriggerType::Temporal => true,
                _ => true,
            }
        }

        fn evaluation_dimensions(&self) -> Vec<EvaluationDimension> {
            vec![
                EvaluationDimension::Economic,
                EvaluationDimension::Environmental,
                EvaluationDimension::Safety,
                EvaluationDimension::Functional,
                EvaluationDimension::Aesthetic,
            ]
        }
    }

    // Simple implementations for required traits
    #[derive(Debug)]
    struct CarAsset {
        id: AssetId,
        name: String,
        properties: HashMap<String, f64>,
    }

    impl PhysicalAsset for CarAsset {
        fn asset_id(&self) -> &AssetId { &self.id }
        fn name(&self) -> &str { &self.name }
        fn physical_properties(&self) -> HashMap<String, f64> { self.properties.clone() }
        fn performance_characteristics(&self) -> HashMap<String, f64> { HashMap::new() }
        fn economic_attributes(&self) -> HashMap<String, f64> { HashMap::new() }
        fn environmental_impact(&self) -> HashMap<String, f64> { HashMap::new() }
        fn is_available(&self, _time: SimulationTime) -> bool { true }
        fn update_state(&mut self, _time: SimulationTime) -> Result<()> { Ok(()) }
    }

    #[derive(Debug)]
    struct MarketInfo {
        id: AssetId,
        content: String,
    }

    impl KnowledgeAsset for MarketInfo {
        fn asset_id(&self) -> &AssetId { &self.id }
        fn content(&self) -> &str { &self.content }
        fn reliability(&self) -> f64 { 0.8 }
        fn relevance(&self, _topic: &str) -> f64 { 0.7 }
        fn timestamp(&self) -> SimulationTime { 0.0 }
        fn is_accessible_to(&self, _agent_id: &AgentId) -> bool { true }
        fn metadata(&self) -> HashMap<String, String> { HashMap::new() }
        fn update_reliability(&mut self, _new_reliability: f64) -> Result<()> { Ok(()) }
    }

    #[derive(Debug)]
    struct SocialNetwork {
        agents: Vec<AgentId>,
        connections: HashMap<AgentId, Vec<AgentId>>,
    }

    impl SocialNetwork {
        fn new() -> Self {
            Self {
                agents: Vec::new(),
                connections: HashMap::new(),
            }
        }
    }

    impl Network for SocialNetwork {
        fn agents(&self) -> Vec<AgentId> { self.agents.clone() }
        fn are_connected(&self, agent1: &AgentId, agent2: &AgentId) -> bool {
            self.connections.get(agent1).map_or(false, |neighbors| neighbors.contains(agent2))
        }
        fn connection_strength(&self, agent1: &AgentId, agent2: &AgentId) -> f64 {
            if self.are_connected(agent1, agent2) { 0.8 } else { 0.0 }
        }
        fn add_agent(&mut self, agent_id: AgentId) -> Result<()> {
            if !self.agents.contains(&agent_id) {
                self.agents.push(agent_id.clone());
                self.connections.insert(agent_id, Vec::new());
            }
            Ok(())
        }
        fn remove_agent(&mut self, agent_id: &AgentId) -> Result<()> {
            self.agents.retain(|id| id != agent_id);
            self.connections.remove(agent_id);
            Ok(())
        }
        fn connect_agents(&mut self, agent1: AgentId, agent2: AgentId, _strength: f64) -> Result<()> {
            self.connections.entry(agent1.clone()).or_insert_with(Vec::new).push(agent2.clone());
            self.connections.entry(agent2).or_insert_with(Vec::new).push(agent1);
            Ok(())
        }
        fn neighbors(&self, agent_id: &AgentId) -> Vec<AgentId> {
            self.connections.get(agent_id).cloned().unwrap_or_default()
        }
        fn network_statistics(&self) -> consumer_choice_metamodel::environment::NetworkStatistics {
            let connection_count = self.connections.values().map(|v| v.len()).sum::<usize>() / 2;
            let avg_degree = if self.agents.is_empty() { 0.0 } else { 2.0 * connection_count as f64 / self.agents.len() as f64 };

            consumer_choice_metamodel::environment::NetworkStatistics {
                agent_count: self.agents.len(),
                connection_count,
                average_degree: avg_degree,
                clustering_coefficient: 0.3, // Simplified
                network_density: if self.agents.len() > 1 {
                    2.0 * connection_count as f64 / (self.agents.len() * (self.agents.len() - 1)) as f64
                } else { 0.0 },
            }
        }
    }

    #[derive(Debug)]
    struct SimpleRules;

    #[cfg_attr(feature = "async", async_trait)]
    impl RulesOfInteraction for SimpleRules {
        type Interaction = String;

        #[cfg(feature = "async")]
        async fn is_interaction_allowed(&self, _initiator: &AgentId, _target: &AgentId, _interaction: &Self::Interaction, _time: SimulationTime) -> Result<bool> { Ok(true) }

        #[cfg(not(feature = "async"))]
        fn is_interaction_allowed(&self, _initiator: &AgentId, _target: &AgentId, _interaction: &Self::Interaction, _time: SimulationTime) -> Result<bool> { Ok(true) }

        #[cfg(feature = "async")]
        async fn process_interaction(&self, _initiator: &AgentId, target: &AgentId, _interaction: Self::Interaction, _time: SimulationTime) -> Result<Vec<consumer_choice_metamodel::environment::InteractionEffect>> {
            Ok(vec![consumer_choice_metamodel::environment::InteractionEffect {
                target_agent: target.clone(),
                effect_type: "influence".to_string(),
                magnitude: 0.1,
                duration: Some(5.0),
            }])
        }

        #[cfg(not(feature = "async"))]
        fn process_interaction(&self, _initiator: &AgentId, target: &AgentId, _interaction: Self::Interaction, _time: SimulationTime) -> Result<Vec<consumer_choice_metamodel::environment::InteractionEffect>> {
            Ok(vec![consumer_choice_metamodel::environment::InteractionEffect {
                target_agent: target.clone(),
                effect_type: "influence".to_string(),
                magnitude: 0.1,
                duration: Some(5.0),
            }])
        }

        fn interaction_cost(&self, _interaction: &Self::Interaction) -> f64 { 0.1 }
    }

    #[derive(Debug)]
    struct EconomicCycle {
        cycle_length: f64,
        amplitude: f64,
    }

    impl EconomicCycle {
        fn new() -> Self {
            Self {
                cycle_length: 20.0,
                amplitude: 0.3,
            }
        }
    }

    #[cfg_attr(feature = "async", async_trait)]
    impl ExogenousProcess for EconomicCycle {
        #[cfg(feature = "async")]
        async fn update_environment(&self, time: SimulationTime) -> Result<Vec<consumer_choice_metamodel::environment::EnvironmentChange>> {
            let phase = (time * 2.0 * std::f64::consts::PI / self.cycle_length).sin();
            let economic_impact = phase * self.amplitude;

            Ok(vec![consumer_choice_metamodel::environment::EnvironmentChange {
                change_type: "economic_conditions".to_string(),
                affected_assets: Vec::new(),
                magnitude: economic_impact,
                duration: Some(1.0),
                description: format!("Economic cycle impact: {:.2}", economic_impact),
            }])
        }

        #[cfg(not(feature = "async"))]
        fn update_environment(&self, time: SimulationTime) -> Result<Vec<consumer_choice_metamodel::environment::EnvironmentChange>> {
            let phase = (time * 2.0 * std::f64::consts::PI / self.cycle_length).sin();
            let economic_impact = phase * self.amplitude;

            Ok(vec![consumer_choice_metamodel::environment::EnvironmentChange {
                change_type: "economic_conditions".to_string(),
                affected_assets: Vec::new(),
                magnitude: economic_impact,
                duration: Some(1.0),
                description: format!("Economic cycle impact: {:.2}", economic_impact),
            }])
        }

        fn is_active(&self, _time: SimulationTime) -> bool { true }
        fn name(&self) -> &str { "Economic Cycle" }
        fn frequency(&self) -> f64 { 1.0 / self.cycle_length }
    }

    // Custom information filter for vehicle information
    #[derive(Debug)]
    struct VehicleInfoFilter {
        min_relevance: f64,
    }

    impl VehicleInfoFilter {
        fn new(min_relevance: f64) -> Self {
            Self { min_relevance }
        }
    }

    #[cfg_attr(feature = "async", async_trait)]
    impl InformationFilter for VehicleInfoFilter {
        #[cfg(feature = "async")]
        async fn filter_information(&self, information: Vec<Information>, _agent_id: &AgentId, _context: &FilterContext) -> Result<Vec<Information>> {
            Ok(information.into_iter()
                .filter(|info| info.topic.contains("vehicle") || info.topic.contains("car"))
                .collect())
        }

        #[cfg(not(feature = "async"))]
        fn filter_information(&self, information: Vec<Information>, _agent_id: &AgentId, _context: &FilterContext) -> Result<Vec<Information>> {
            Ok(information.into_iter()
                .filter(|info| info.topic.contains("vehicle") || info.topic.contains("car"))
                .collect())
        }

        #[cfg(feature = "async")]
        async fn passes_filter(&self, information: &Information, _agent_id: &AgentId, _context: &FilterContext) -> Result<bool> {
            Ok(information.topic.contains("vehicle") || information.topic.contains("car"))
        }

        #[cfg(not(feature = "async"))]
        fn passes_filter(&self, information: &Information, _agent_id: &AgentId, _context: &FilterContext) -> Result<bool> {
            Ok(information.topic.contains("vehicle") || information.topic.contains("car"))
        }

        fn filter_name(&self) -> &str { "VehicleInfoFilter" }
        fn parameters(&self) -> HashMap<String, f64> {
            let mut params = HashMap::new();
            params.insert("min_relevance".to_string(), self.min_relevance);
            params
        }
    }

    // Brand preference distorter
    #[derive(Debug)]
    struct BrandPreferenceDistorter {
        preferred_brands: HashMap<String, f64>,
    }

    impl BrandPreferenceDistorter {
        fn new() -> Self {
            let mut preferred_brands = HashMap::new();
            preferred_brands.insert("Toyota".to_string(), 0.8);
            preferred_brands.insert("Honda".to_string(), 0.7);
            preferred_brands.insert("BMW".to_string(), 0.6);

            Self { preferred_brands }
        }
    }

    #[cfg_attr(feature = "async", async_trait)]
    impl InformationDistorter for BrandPreferenceDistorter {
        #[cfg(feature = "async")]
        async fn distort_information(&self, mut information: Information, _agent_id: &AgentId, _context: &DistortionContext) -> Result<Information> {
            // Boost reliability for preferred brands
            for (brand, boost) in &self.preferred_brands {
                if information.content.contains(brand) {
                    information.reliability = (information.reliability + boost * 0.1).min(1.0);
                    break;
                }
            }
            Ok(information)
        }

        #[cfg(not(feature = "async"))]
        fn distort_information(&self, mut information: Information, _agent_id: &AgentId, _context: &DistortionContext) -> Result<Information> {
            // Boost reliability for preferred brands
            for (brand, boost) in &self.preferred_brands {
                if information.content.contains(brand) {
                    information.reliability = (information.reliability + boost * 0.1).min(1.0);
                    break;
                }
            }
            Ok(information)
        }

        fn distortion_magnitude(&self, information: &Information, _agent_id: &AgentId) -> f64 {
            for (brand, _) in &self.preferred_brands {
                if information.content.contains(brand) {
                    return 0.1;
                }
            }
            0.0
        }

        fn distorter_name(&self) -> &str { "BrandPreferenceDistorter" }
        fn parameters(&self) -> HashMap<String, f64> {
            self.preferred_brands.clone()
        }
    }

    // Start simulation
    println!("\n1. Setting up simulation environment...");

    // Create environment
    let mut environment: Environment<CarAsset, MarketInfo, SocialNetwork, SimpleRules, EconomicCycle> = Environment::new(SimpleRules);

    // Add some physical assets (vehicles)
    let vehicles = vec![
        CarAsset {
            id: AssetId::new(),
            name: "Toyota Prius".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("price".to_string(), 28000.0);
                props.insert("fuel_efficiency".to_string(), 45.0);
                props
            }
        },
        CarAsset {
            id: AssetId::new(),
            name: "BMW X5".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("price".to_string(), 55000.0);
                props.insert("fuel_efficiency".to_string(), 25.0);
                props
            }
        },
    ];

    for vehicle in vehicles {
        environment.add_physical_asset(vehicle)?;
    }

    // Add knowledge assets
    let market_reports = vec![
        MarketInfo {
            id: AssetId::new(),
            content: "Electric vehicle sales increased 40% this year".to_string(),
        },
        MarketInfo {
            id: AssetId::new(),
            content: "Fuel prices expected to rise due to global tensions".to_string(),
        },
    ];

    for report in market_reports {
        environment.add_knowledge_asset(report)?;
    }

    // Add social network
    let mut social_network = SocialNetwork::new();
    environment.add_network(social_network);

    // Add exogenous process
    environment.add_exogenous_process(EconomicCycle::new());

    println!("Environment setup complete");

    // Create information transformer
    println!("\n2. Setting up information processing...");
    let mut transformer: Transformer<VehicleInfoFilter, BrandPreferenceDistorter> = Transformer::new(50.0);
    transformer.add_filter(VehicleInfoFilter::new(0.5));
    transformer.add_distorter(BrandPreferenceDistorter::new());
    println!("Information transformer configured");

    // Create model configuration
    println!("\n3. Creating model configuration...");
    let config = ModelConfiguration::new(
        "Vehicle Purchase Simulation".to_string(),
        "Simulation of consumer vehicle purchase decisions over time".to_string(),
    )
    .with_time_step(1.0)
    .with_max_time(50.0)
    .with_random_seed(123)
    .with_validation(true);

    println!("Model configured: {} steps over {:.0} time units",
             (config.max_simulation_time / config.time_step) as i32,
             config.max_simulation_time);

    // Create model
    println!("\n4. Creating simulation model...");
    let mut model: ConsumerChoiceModel<
        BasicAgentAttributes,
        VehicleChoiceModule,
        CarAsset,
        MarketInfo,
        SocialNetwork,
        SimpleRules,
        EconomicCycle,
        VehicleInfoFilter,
        BrandPreferenceDistorter,
    > = ConsumerChoiceModel::new(config, environment, transformer);

    // Add event handler
    #[derive(Debug)]
    struct SimulationEventHandler;
    impl consumer_choice_metamodel::utils::EventHandler for SimulationEventHandler {
        fn handle_event(&self, event: &ModelEvent) {
            match event.event_type {
                EventType::SimulationStarted => println!("Simulation started"),
                EventType::AgentAdded => println!("Agent added: {}", event.description),
                EventType::ChoiceMade => println!("Choice made: {}", event.description),
                EventType::SimulationCompleted => println!("🏁 Simulation completed"),
                _ => {} // Suppress other events for cleaner output
            }
        }
    }
    model.event_bus().add_handler(Box::new(SimulationEventHandler));

    println!("Model created with event handling");

    // Create diverse agents
    println!("\n5. Creating diverse agent population...");
    let agent_profiles = vec![
        ("Young Professional", 35000.0, 28.0, DecisionStyle::Economic),
        ("Family Oriented", 75000.0, 40.0, DecisionStyle::Analytical),
        ("Status Seeker", 120000.0, 45.0, DecisionStyle::Emotional),
        ("Environmentalist", 60000.0, 35.0, DecisionStyle::Analytical),
        ("Social Influencer", 85000.0, 30.0, DecisionStyle::Social),
    ];

    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    for (profile_name, income, age, decision_style) in agent_profiles {
        let agent_id = AgentId::new();

        // Create varied psychological attributes
        let price_sensitivity = rng.gen_range(0.3..0.9);
        let environmental_concern = rng.gen_range(0.1..0.9);
        let safety_concern = rng.gen_range(0.4..0.9);
        let status_seeking = rng.gen_range(0.1..0.8);

        let attributes = BasicAgentAttributes::new(agent_id.clone())
            .with_psychological_attribute("price_sensitivity".to_string(), price_sensitivity)
            .with_psychological_attribute("environmental_concern".to_string(), environmental_concern)
            .with_psychological_attribute("safety_concern".to_string(), safety_concern)
            .with_psychological_attribute("status_seeking".to_string(), status_seeking)
            .with_socioeconomic_attribute("income".to_string(), income)
            .with_socioeconomic_attribute("age".to_string(), age)
            .with_stock_variable("current_vehicle".to_string(), Some("old_car".to_string()));

        let choice_module = VehicleChoiceModule::new(rng.gen_range(0.2..0.8), decision_style);
        let agent = ConsumerAgent::new(attributes, choice_module);

        model.add_agent(agent)?;
        println!("Added {}: Income=${:.0}, Age={:.0}, Price Sens={:.2}, Env Concern={:.2}",
                 profile_name, income, age, price_sensitivity, environmental_concern);
    }

    // Display initial statistics
    println!("\n6. Initial model statistics:");
    let stats = model.statistics();
    println!("   Agents: {}", stats.total_agents);
    println!("   Choices made: {}", stats.total_choices_made);

    // Run simulation
    println!("\n7. Running simulation...");
    println!("   Progress: [", );
    let start_time = std::time::Instant::now();

    #[cfg(not(feature = "async"))]
    {
        model.start()?;
        let mut step_count = 0;
        let total_steps = (model.configuration().max_simulation_time / model.configuration().time_step) as i32;

        while model.state() == consumer_choice_metamodel::model::ModelState::Running {
            model.step()?;
            step_count += 1;

            // Show progress
            if step_count % 10 == 0 {
                print!("=");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
    }

    #[cfg(feature = "async")]
    {
        use tokio;
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            model.start().unwrap();
            let mut step_count = 0;

            while model.state() == consumer_choice_metamodel::model::ModelState::Running {
                model.step().await.unwrap();
                step_count += 1;

                if step_count % 10 == 0 {
                    print!("=");
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }
            }
        });
    }

    let simulation_duration = start_time.elapsed();
    println!("]");
    println!("Simulation completed in {:.2?}", simulation_duration);

    // Analyze results
    println!("\n8. Simulation Results:");
    let final_stats = model.statistics();
    println!("   Total simulation time: {:.2}", final_stats.simulation_duration);
    println!("   Total choices made: {}", final_stats.total_choices_made);
    println!("   Average choices per agent: {:.2}", final_stats.average_choices_per_agent);
    println!("   Events processed: {}", final_stats.events_processed);

    // Analyze agent choices
    println!("\n9. Agent Choice Analysis:");
    for agent_id in model.agent_ids() {
        if let Some(agent) = model.get_agent(&agent_id) {
            let choice_count = agent.choice_history().len();
            let last_choice_time = agent.last_choice_time().unwrap_or(0.0);

            println!("   Agent {}: {} choices, last choice at time {:.1}",
                     agent_id.to_string().chars().take(8).collect::<String>(),
                     choice_count,
                     last_choice_time);
        }
    }

    // Event analysis
    println!("\n10. Event Analysis:");
    let events = model.event_bus().get_events();
    let choice_events = model.event_bus().get_events_of_type(EventType::ChoiceMade);

    println!("    Total events: {}", events.len());
    println!("    Choice events: {}", choice_events.len());

    if !choice_events.is_empty() {
        println!("    First choice at: {:.1}", choice_events[0].timestamp);
        println!("    Last choice at: {:.1}", choice_events.last().unwrap().timestamp);
    }

    println!("\n==============================================");
    println!("Vehicle Purchase Simulation Complete!");
    println!("==============================================");

    Ok(())
}