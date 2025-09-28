//! Agent-related traits and types for the Consumer Choice Metamodel

use crate::types::{AgentId, EvaluationDimension, SimulationTime, TriggerType};
use crate::{Error, Result};
use std::collections::HashMap;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Trait defining the attributes and characteristics of an agent
pub trait AgentAttributes: std::fmt::Debug + Send + Sync {
    /// Get the unique identifier for this agent
    fn agent_id(&self) -> &AgentId;

    /// Get psychological attributes (e.g., risk aversion, environmental concern)
    /// Returns a map of attribute names to their values (typically 0.0 to 1.0)
    fn psychological_attributes(&self) -> HashMap<String, f64>;

    /// Get socioeconomic attributes (e.g., income, age, education level)
    /// Returns a map of attribute names to their values
    fn socioeconomic_attributes(&self) -> HashMap<String, f64>;

    /// Get stock variables representing current possessions/states
    /// Returns a map of variable names to their current values (None means not owned)
    fn stock_variables(&self) -> HashMap<String, Option<String>>;

    /// Update agent attributes based on changes
    fn update_attributes(&mut self, changes: HashMap<String, f64>) -> Result<()>;

    /// Get a specific psychological attribute by name
    fn get_psychological_attribute(&self, name: &str) -> Option<f64> {
        self.psychological_attributes().get(name).copied()
    }

    /// Get a specific socioeconomic attribute by name
    fn get_socioeconomic_attribute(&self, name: &str) -> Option<f64> {
        self.socioeconomic_attributes().get(name).copied()
    }

    /// Check if agent owns a specific stock variable
    fn owns_stock_variable(&self, name: &str) -> bool {
        self.stock_variables()
            .get(name)
            .map(|v| v.is_some())
            .unwrap_or(false)
    }
}

/// Trait defining the decision-making logic for agents
#[cfg_attr(feature = "async", async_trait)]
pub trait ChoiceModule: std::fmt::Debug + Send + Sync {
    /// Type representing a choice option
    type Choice: Clone + std::fmt::Debug + Send + Sync;

    /// Type representing the context in which choices are made
    type Context: std::fmt::Debug + Send + Sync;

    /// Evaluate available choices and select one
    #[cfg(feature = "async")]
    async fn make_choice(
        &self,
        choices: Vec<Self::Choice>,
        context: &Self::Context,
        trigger: TriggerType,
    ) -> Result<Option<Self::Choice>>;

    #[cfg(not(feature = "async"))]
    fn make_choice(
        &self,
        choices: Vec<Self::Choice>,
        context: &Self::Context,
        trigger: TriggerType,
    ) -> Result<Option<Self::Choice>>;

    /// Evaluate a single choice across multiple dimensions
    #[cfg(feature = "async")]
    async fn evaluate_choice(
        &self,
        choice: &Self::Choice,
        dimensions: &[EvaluationDimension],
        context: &Self::Context,
    ) -> Result<HashMap<EvaluationDimension, f64>>;

    #[cfg(not(feature = "async"))]
    fn evaluate_choice(
        &self,
        choice: &Self::Choice,
        dimensions: &[EvaluationDimension],
        context: &Self::Context,
    ) -> Result<HashMap<EvaluationDimension, f64>>;

    /// Determine if the agent should make a choice given a trigger
    fn should_make_choice(&self, trigger: TriggerType, context: &Self::Context) -> bool;

    /// Get the evaluation dimensions this choice module considers
    fn evaluation_dimensions(&self) -> Vec<EvaluationDimension>;
}

/// Main consumer agent implementation
#[derive(Debug)]
pub struct ConsumerAgent<A, C>
where
    A: AgentAttributes,
    C: ChoiceModule,
{
    attributes: A,
    choice_module: C,
    last_choice_time: Option<SimulationTime>,
    choice_history: Vec<ChoiceRecord<C::Choice>>,
}

/// Record of a choice made by an agent
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChoiceRecord<T> {
    pub choice: T,
    pub time: SimulationTime,
    pub trigger: TriggerType,
    pub evaluation_scores: HashMap<EvaluationDimension, f64>,
}

impl<A, C> ConsumerAgent<A, C>
where
    A: AgentAttributes,
    C: ChoiceModule,
{
    /// Create a new consumer agent
    pub fn new(attributes: A, choice_module: C) -> Self {
        Self {
            attributes,
            choice_module,
            last_choice_time: None,
            choice_history: Vec::new(),
        }
    }

    /// Get the agent's attributes
    pub fn attributes(&self) -> &A {
        &self.attributes
    }

    /// Get mutable reference to the agent's attributes
    pub fn attributes_mut(&mut self) -> &mut A {
        &mut self.attributes
    }

    /// Get the agent's choice module
    pub fn choice_module(&self) -> &C {
        &self.choice_module
    }

    /// Get the agent's choice history
    pub fn choice_history(&self) -> &[ChoiceRecord<C::Choice>] {
        &self.choice_history
    }

    /// Get the time of the last choice made
    pub fn last_choice_time(&self) -> Option<SimulationTime> {
        self.last_choice_time
    }

    /// Process a trigger and potentially make a choice
    #[cfg(feature = "async")]
    pub async fn process_trigger(
        &mut self,
        trigger: TriggerType,
        choices: Vec<C::Choice>,
        context: &C::Context,
        current_time: SimulationTime,
    ) -> Result<Option<C::Choice>> {
        if !self.choice_module.should_make_choice(trigger, context) {
            return Ok(None);
        }

        let chosen = self.choice_module.make_choice(choices, context, trigger).await?;

        if let Some(ref choice) = chosen {
            // Evaluate the chosen option
            let dimensions = self.choice_module.evaluation_dimensions();
            let evaluation_scores = self
                .choice_module
                .evaluate_choice(choice, &dimensions, context)
                .await?;

            // Record the choice
            let record = ChoiceRecord {
                choice: choice.clone(),
                time: current_time,
                trigger,
                evaluation_scores,
            };

            self.choice_history.push(record);
            self.last_choice_time = Some(current_time);
        }

        Ok(chosen)
    }

    #[cfg(not(feature = "async"))]
    pub fn process_trigger(
        &mut self,
        trigger: TriggerType,
        choices: Vec<C::Choice>,
        context: &C::Context,
        current_time: SimulationTime,
    ) -> Result<Option<C::Choice>> {
        if !self.choice_module.should_make_choice(trigger.clone(), context) {
            return Ok(None);
        }

        let chosen = self.choice_module.make_choice(choices, context, trigger.clone())?;

        if let Some(ref choice) = chosen {
            // Evaluate the chosen option
            let dimensions = self.choice_module.evaluation_dimensions();
            let evaluation_scores = self
                .choice_module
                .evaluate_choice(choice, &dimensions, context)?;

            // Record the choice
            let record = ChoiceRecord {
                choice: choice.clone(),
                time: current_time,
                trigger,
                evaluation_scores,
            };

            self.choice_history.push(record);
            self.last_choice_time = Some(current_time);
        }

        Ok(chosen)
    }

    /// Clear the choice history
    pub fn clear_history(&mut self) {
        self.choice_history.clear();
        self.last_choice_time = None;
    }

    /// Get choices made within a time range
    pub fn choices_in_time_range(
        &self,
        start_time: SimulationTime,
        end_time: SimulationTime,
    ) -> Vec<&ChoiceRecord<C::Choice>> {
        self.choice_history
            .iter()
            .filter(|record| record.time >= start_time && record.time <= end_time)
            .collect()
    }

    /// Get the most recent choice
    pub fn most_recent_choice(&self) -> Option<&ChoiceRecord<C::Choice>> {
        self.choice_history.last()
    }
}

/// Simple implementation of AgentAttributes for basic use cases
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BasicAgentAttributes {
    agent_id: AgentId,
    psychological: HashMap<String, f64>,
    socioeconomic: HashMap<String, f64>,
    stock_variables: HashMap<String, Option<String>>,
}

impl BasicAgentAttributes {
    /// Create new basic agent attributes
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            psychological: HashMap::new(),
            socioeconomic: HashMap::new(),
            stock_variables: HashMap::new(),
        }
    }

    /// Add a psychological attribute
    pub fn with_psychological_attribute(mut self, name: String, value: f64) -> Self {
        self.psychological.insert(name, value);
        self
    }

    /// Add a socioeconomic attribute
    pub fn with_socioeconomic_attribute(mut self, name: String, value: f64) -> Self {
        self.socioeconomic.insert(name, value);
        self
    }

    /// Add a stock variable
    pub fn with_stock_variable(mut self, name: String, value: Option<String>) -> Self {
        self.stock_variables.insert(name, value);
        self
    }
}

impl AgentAttributes for BasicAgentAttributes {
    fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }

    fn psychological_attributes(&self) -> HashMap<String, f64> {
        self.psychological.clone()
    }

    fn socioeconomic_attributes(&self) -> HashMap<String, f64> {
        self.socioeconomic.clone()
    }

    fn stock_variables(&self) -> HashMap<String, Option<String>> {
        self.stock_variables.clone()
    }

    fn update_attributes(&mut self, changes: HashMap<String, f64>) -> Result<()> {
        for (key, value) in changes {
            if self.psychological.contains_key(&key) {
                self.psychological.insert(key, value);
            } else if self.socioeconomic.contains_key(&key) {
                self.socioeconomic.insert(key, value);
            } else {
                return Err(Error::Agent(format!("Unknown attribute: {}", key)));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestChoice {
        name: String,
        value: f64,
    }

    #[derive(Debug)]
    struct TestContext {
        available_budget: f64,
    }

    #[derive(Debug)]
    struct TestChoiceModule;

    #[cfg_attr(feature = "async", async_trait)]
    impl ChoiceModule for TestChoiceModule {
        type Choice = TestChoice;
        type Context = TestContext;

        #[cfg(feature = "async")]
        async fn make_choice(
            &self,
            choices: Vec<Self::Choice>,
            _context: &Self::Context,
            _trigger: TriggerType,
        ) -> Result<Option<Self::Choice>> {
            Ok(choices.into_iter().next())
        }

        #[cfg(not(feature = "async"))]
        fn make_choice(
            &self,
            choices: Vec<Self::Choice>,
            _context: &Self::Context,
            _trigger: TriggerType,
        ) -> Result<Option<Self::Choice>> {
            Ok(choices.into_iter().next())
        }

        #[cfg(feature = "async")]
        async fn evaluate_choice(
            &self,
            choice: &Self::Choice,
            _dimensions: &[EvaluationDimension],
            _context: &Self::Context,
        ) -> Result<HashMap<EvaluationDimension, f64>> {
            let mut scores = HashMap::new();
            scores.insert(EvaluationDimension::Economic, choice.value);
            Ok(scores)
        }

        #[cfg(not(feature = "async"))]
        fn evaluate_choice(
            &self,
            choice: &Self::Choice,
            _dimensions: &[EvaluationDimension],
            _context: &Self::Context,
        ) -> Result<HashMap<EvaluationDimension, f64>> {
            let mut scores = HashMap::new();
            scores.insert(EvaluationDimension::Economic, choice.value);
            Ok(scores)
        }

        fn should_make_choice(&self, _trigger: TriggerType, _context: &Self::Context) -> bool {
            true
        }

        fn evaluation_dimensions(&self) -> Vec<EvaluationDimension> {
            vec![EvaluationDimension::Economic]
        }
    }

    #[test]
    fn test_basic_agent_attributes() {
        let agent_id = AgentId::new();
        let attrs = BasicAgentAttributes::new(agent_id.clone())
            .with_psychological_attribute("risk_aversion".to_string(), 0.5)
            .with_socioeconomic_attribute("income".to_string(), 50000.0)
            .with_stock_variable("car".to_string(), Some("sedan".to_string()));

        assert_eq!(attrs.agent_id(), &agent_id);
        assert_eq!(attrs.get_psychological_attribute("risk_aversion"), Some(0.5));
        assert_eq!(attrs.get_socioeconomic_attribute("income"), Some(50000.0));
        assert!(attrs.owns_stock_variable("car"));
        assert!(!attrs.owns_stock_variable("house"));
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn test_consumer_agent() {
        let agent_id = AgentId::new();
        let attrs = BasicAgentAttributes::new(agent_id);
        let choice_module = TestChoiceModule;
        let mut agent = ConsumerAgent::new(attrs, choice_module);

        let choices = vec![TestChoice {
            name: "choice1".to_string(),
            value: 10.0,
        }];
        let context = TestContext {
            available_budget: 1000.0,
        };

        let result = agent
            .process_trigger(TriggerType::Economic, choices, &context, 1.0)
            .unwrap();

        assert!(result.is_some());
        assert_eq!(agent.choice_history().len(), 1);
        assert_eq!(agent.last_choice_time(), Some(1.0));
    }
}