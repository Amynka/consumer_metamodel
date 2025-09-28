//! Information processing traits and types for the Consumer Choice Metamodel

use crate::types::{AgentId, SimulationTime};
use crate::Result;
use std::collections::HashMap;

#[cfg(feature = "async")]
use async_trait::async_trait;

/// Represents a piece of information in the system
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Information {
    pub content: String,
    pub source: AgentId,
    pub timestamp: SimulationTime,
    pub reliability: f64,
    pub topic: String,
    pub metadata: HashMap<String, String>,
}

impl Information {
    /// Create new information
    pub fn new(
        content: String,
        source: AgentId,
        timestamp: SimulationTime,
        reliability: f64,
        topic: String,
    ) -> Self {
        Self {
            content,
            source,
            timestamp,
            reliability,
            topic,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the information
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if the information is recent (within a time threshold)
    pub fn is_recent(&self, current_time: SimulationTime, threshold: SimulationTime) -> bool {
        current_time - self.timestamp <= threshold
    }

    /// Get the age of the information
    pub fn age(&self, current_time: SimulationTime) -> SimulationTime {
        current_time - self.timestamp
    }
}

/// Trait for filtering information based on various criteria
#[cfg_attr(feature = "async", async_trait)]
pub trait InformationFilter: std::fmt::Debug + Send + Sync {
    /// Filter a list of information items
    #[cfg(feature = "async")]
    async fn filter_information(
        &self,
        information: Vec<Information>,
        agent_id: &AgentId,
        context: &FilterContext,
    ) -> Result<Vec<Information>>;

    #[cfg(not(feature = "async"))]
    fn filter_information(
        &self,
        information: Vec<Information>,
        agent_id: &AgentId,
        context: &FilterContext,
    ) -> Result<Vec<Information>>;

    /// Check if a single piece of information passes the filter
    #[cfg(feature = "async")]
    async fn passes_filter(
        &self,
        information: &Information,
        agent_id: &AgentId,
        context: &FilterContext,
    ) -> Result<bool>;

    #[cfg(not(feature = "async"))]
    fn passes_filter(
        &self,
        information: &Information,
        agent_id: &AgentId,
        context: &FilterContext,
    ) -> Result<bool>;

    /// Get the name/description of this filter
    fn filter_name(&self) -> &str;

    /// Get filter parameters
    fn parameters(&self) -> HashMap<String, f64>;
}

/// Context for information filtering
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FilterContext {
    pub current_time: SimulationTime,
    pub agent_interests: Vec<String>,
    pub relevance_threshold: f64,
    pub reliability_threshold: f64,
    pub recency_threshold: SimulationTime,
    pub max_items: Option<usize>,
}

impl FilterContext {
    /// Create a new filter context with default values
    pub fn new(current_time: SimulationTime) -> Self {
        Self {
            current_time,
            agent_interests: Vec::new(),
            relevance_threshold: 0.5,
            reliability_threshold: 0.3,
            recency_threshold: 100.0,
            max_items: None,
        }
    }

    /// Set agent interests
    pub fn with_interests(mut self, interests: Vec<String>) -> Self {
        self.agent_interests = interests;
        self
    }

    /// Set relevance threshold
    pub fn with_relevance_threshold(mut self, threshold: f64) -> Self {
        self.relevance_threshold = threshold;
        self
    }

    /// Set maximum number of items
    pub fn with_max_items(mut self, max: usize) -> Self {
        self.max_items = Some(max);
        self
    }
}

/// Trait for distorting/biasing information
#[cfg_attr(feature = "async", async_trait)]
pub trait InformationDistorter: std::fmt::Debug + Send + Sync {
    /// Apply distortion to information
    #[cfg(feature = "async")]
    async fn distort_information(
        &self,
        information: Information,
        agent_id: &AgentId,
        context: &DistortionContext,
    ) -> Result<Information>;

    #[cfg(not(feature = "async"))]
    fn distort_information(
        &self,
        information: Information,
        agent_id: &AgentId,
        context: &DistortionContext,
    ) -> Result<Information>;

    /// Calculate the amount of distortion to apply
    fn distortion_magnitude(&self, information: &Information, agent_id: &AgentId) -> f64;

    /// Get the name/description of this distorter
    fn distorter_name(&self) -> &str;

    /// Get distorter parameters
    fn parameters(&self) -> HashMap<String, f64>;
}

/// Context for information distortion
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DistortionContext {
    pub current_time: SimulationTime,
    pub agent_biases: HashMap<String, f64>,
    pub social_influence: f64,
    pub stress_level: f64,
    pub confirmation_bias_strength: f64,
}

impl DistortionContext {
    /// Create a new distortion context
    pub fn new(current_time: SimulationTime) -> Self {
        Self {
            current_time,
            agent_biases: HashMap::new(),
            social_influence: 0.0,
            stress_level: 0.0,
            confirmation_bias_strength: 0.5,
        }
    }

    /// Set agent biases
    pub fn with_biases(mut self, biases: HashMap<String, f64>) -> Self {
        self.agent_biases = biases;
        self
    }

    /// Set social influence level
    pub fn with_social_influence(mut self, influence: f64) -> Self {
        self.social_influence = influence;
        self
    }
}

/// Manages information flow between agents and environment
#[derive(Debug)]
pub struct Transformer<F, D>
where
    F: InformationFilter,
    D: InformationDistorter,
{
    filters: Vec<F>,
    distorters: Vec<D>,
    information_cache: HashMap<AgentId, Vec<Information>>,
    cache_expiry_time: SimulationTime,
}

impl<F, D> Transformer<F, D>
where
    F: InformationFilter,
    D: InformationDistorter,
{
    /// Create a new transformer
    pub fn new(cache_expiry_time: SimulationTime) -> Self {
        Self {
            filters: Vec::new(),
            distorters: Vec::new(),
            information_cache: HashMap::new(),
            cache_expiry_time,
        }
    }

    /// Add a filter to the transformer
    pub fn add_filter(&mut self, filter: F) {
        self.filters.push(filter);
    }

    /// Add a distorter to the transformer
    pub fn add_distorter(&mut self, distorter: D) {
        self.distorters.push(distorter);
    }

    /// Process information for a specific agent
    #[cfg(feature = "async")]
    pub async fn process_information_for_agent(
        &mut self,
        agent_id: &AgentId,
        raw_information: Vec<Information>,
        filter_context: &FilterContext,
        distortion_context: &DistortionContext,
    ) -> Result<Vec<Information>> {
        let mut processed_info = raw_information;

        // Apply filters
        for filter in &self.filters {
            processed_info = filter
                .filter_information(processed_info, agent_id, filter_context)
                .await?;
        }

        // Apply distorters
        let mut distorted_info = Vec::new();
        for info in processed_info {
            let mut distorted = info;
            for distorter in &self.distorters {
                distorted = distorter
                    .distort_information(distorted, agent_id, distortion_context)
                    .await?;
            }
            distorted_info.push(distorted);
        }

        // Cache the result
        self.information_cache
            .insert(agent_id.clone(), distorted_info.clone());

        Ok(distorted_info)
    }

    #[cfg(not(feature = "async"))]
    pub fn process_information_for_agent(
        &mut self,
        agent_id: &AgentId,
        raw_information: Vec<Information>,
        filter_context: &FilterContext,
        distortion_context: &DistortionContext,
    ) -> Result<Vec<Information>> {
        let mut processed_info = raw_information;

        // Apply filters
        for filter in &self.filters {
            processed_info = filter.filter_information(processed_info, agent_id, filter_context)?;
        }

        // Apply distorters
        let mut distorted_info = Vec::new();
        for info in processed_info {
            let mut distorted = info;
            for distorter in &self.distorters {
                distorted =
                    distorter.distort_information(distorted, agent_id, distortion_context)?;
            }
            distorted_info.push(distorted);
        }

        // Cache the result
        self.information_cache
            .insert(agent_id.clone(), distorted_info.clone());

        Ok(distorted_info)
    }

    /// Get cached information for an agent
    pub fn get_cached_information(&self, agent_id: &AgentId) -> Option<&Vec<Information>> {
        self.information_cache.get(agent_id)
    }

    /// Clear expired cache entries
    pub fn clear_expired_cache(&mut self, current_time: SimulationTime) {
        // For simplicity, clear all cache entries if any are expired
        // In a real implementation, you'd track cache timestamps
        if current_time > self.cache_expiry_time {
            self.information_cache.clear();
        }
    }

    /// Get filter count
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }

    /// Get distorter count
    pub fn distorter_count(&self) -> usize {
        self.distorters.len()
    }
}

/// Simple reliability-based filter
#[derive(Debug)]
pub struct ReliabilityFilter {
    min_reliability: f64,
}

impl ReliabilityFilter {
    pub fn new(min_reliability: f64) -> Self {
        Self { min_reliability }
    }
}

#[cfg_attr(feature = "async", async_trait)]
impl InformationFilter for ReliabilityFilter {
    #[cfg(feature = "async")]
    async fn filter_information(
        &self,
        information: Vec<Information>,
        _agent_id: &AgentId,
        _context: &FilterContext,
    ) -> Result<Vec<Information>> {
        Ok(information
            .into_iter()
            .filter(|info| info.reliability >= self.min_reliability)
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
            .filter(|info| info.reliability >= self.min_reliability)
            .collect())
    }

    #[cfg(feature = "async")]
    async fn passes_filter(
        &self,
        information: &Information,
        _agent_id: &AgentId,
        _context: &FilterContext,
    ) -> Result<bool> {
        Ok(information.reliability >= self.min_reliability)
    }

    #[cfg(not(feature = "async"))]
    fn passes_filter(
        &self,
        information: &Information,
        _agent_id: &AgentId,
        _context: &FilterContext,
    ) -> Result<bool> {
        Ok(information.reliability >= self.min_reliability)
    }

    fn filter_name(&self) -> &str {
        "ReliabilityFilter"
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("min_reliability".to_string(), self.min_reliability);
        params
    }
}

/// Simple confirmation bias distorter
#[derive(Debug)]
pub struct ConfirmationBiasDistorter {
    bias_strength: f64,
}

impl ConfirmationBiasDistorter {
    pub fn new(bias_strength: f64) -> Self {
        Self { bias_strength }
    }
}

#[cfg_attr(feature = "async", async_trait)]
impl InformationDistorter for ConfirmationBiasDistorter {
    #[cfg(feature = "async")]
    async fn distort_information(
        &self,
        mut information: Information,
        _agent_id: &AgentId,
        context: &DistortionContext,
    ) -> Result<Information> {
        // Apply confirmation bias by adjusting reliability based on agent biases
        let bias_adjustment = context.confirmation_bias_strength * self.bias_strength;
        information.reliability = (information.reliability + bias_adjustment).clamp(0.0, 1.0);
        Ok(information)
    }

    #[cfg(not(feature = "async"))]
    fn distort_information(
        &self,
        mut information: Information,
        _agent_id: &AgentId,
        context: &DistortionContext,
    ) -> Result<Information> {
        // Apply confirmation bias by adjusting reliability based on agent biases
        let bias_adjustment = context.confirmation_bias_strength * self.bias_strength;
        information.reliability = (information.reliability + bias_adjustment).clamp(0.0, 1.0);
        Ok(information)
    }

    fn distortion_magnitude(&self, _information: &Information, _agent_id: &AgentId) -> f64 {
        self.bias_strength
    }

    fn distorter_name(&self) -> &str {
        "ConfirmationBiasDistorter"
    }

    fn parameters(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("bias_strength".to_string(), self.bias_strength);
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_information_creation() {
        let agent_id = AgentId::new();
        let info = Information::new(
            "Test content".to_string(),
            agent_id,
            10.0,
            0.8,
            "test_topic".to_string(),
        );

        assert_eq!(info.content, "Test content");
        assert_eq!(info.reliability, 0.8);
        assert_eq!(info.topic, "test_topic");
    }

    #[test]
    fn test_information_age() {
        let agent_id = AgentId::new();
        let info = Information::new(
            "Test content".to_string(),
            agent_id,
            10.0,
            0.8,
            "test_topic".to_string(),
        );

        assert_eq!(info.age(20.0), 10.0);
        assert!(info.is_recent(15.0, 10.0));
        assert!(!info.is_recent(25.0, 10.0));
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn test_reliability_filter() {
        let filter = ReliabilityFilter::new(0.5);
        let agent_id = AgentId::new();

        let high_reliability_info = Information::new(
            "Reliable content".to_string(),
            agent_id.clone(),
            0.0,
            0.8,
            "topic".to_string(),
        );

        let low_reliability_info = Information::new(
            "Unreliable content".to_string(),
            agent_id.clone(),
            0.0,
            0.3,
            "topic".to_string(),
        );

        let context = FilterContext::new(0.0);

        let result = filter
            .filter_information(
                vec![high_reliability_info, low_reliability_info],
                &agent_id,
                &context,
            )
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "Reliable content");
    }

    #[test]
    fn test_transformer_creation() {
        let transformer: Transformer<ReliabilityFilter, ConfirmationBiasDistorter> =
            Transformer::new(100.0);

        assert_eq!(transformer.filter_count(), 0);
        assert_eq!(transformer.distorter_count(), 0);
    }
}
