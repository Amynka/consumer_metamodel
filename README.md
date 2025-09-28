# Consumer Choice Metamodel


## Quick setup run

```bash
cargo check                    # Quick compilation check
cargo build                    # Build library
cargo test                     # Run tests
cargo run --example basic_usage                    # Run basic example
cargo run --example simulation --features simulation  # Run simulation
cargo run --example adoption --features simulation  # Run simulation
```

## File Content

### Root Files
- **Cargo.toml**: Project configuration with features (serde, async, simulation)
- **README.md**: Comprehensive documentation with installation and usage

### Source Files (`src/`)
- **lib.rs**: Main library entry point with public API and error types
- **types.rs**: Core types (AgentId, TriggerType, EvaluationDimension, etc.)
- **agent.rs**: Agent traits and ConsumerAgent implementation
- **environment.rs**: Environment, assets, networks, and processes
- **information.rs**: Information processing, filtering, and distortion
- **model.rs**: Main ConsumerChoiceModel with simulation logic
- **factory.rs**: Factory pattern for creating model components
- **utils.rs**: Validation, event system, and utilities

### Examples (`examples/`)
- **basic_usage.rs**: Simple example showing core concepts
- **simulation.rs**: Complete vehicle purchase simulation

## Features and Capabilities

### Core Features
- **Agent-based modeling** with customizable attributes and decision-making
- **Environment simulation** with physical assets, knowledge, and networks
- **Information processing** with filtering and bias modeling
- **Event system** for monitoring and analysis
- **Validation utilities** for model integrity
- **Factory pattern** for flexible component creation

### Optional Features (via Cargo features)
- `serde` - Serialization support for saving/loading models
- `async` - Async/await support for non-blocking operations  
- `simulation` - Random number generation for stochastic models
- `full` - All features enabled


## Testing the Project

```bash
# Basic functionality tests
cargo test

# Run examples
cargo run --example basic_usage
cargo run --example simulation --features simulation

# Performance benchmarks (if implemented)
cargo bench
```

##  Example Usage

### Simple Agent Creation
```rust
use consumer_choice_metamodel::*;

let agent_id = AgentId::new();
let attributes = BasicAgentAttributes::new(agent_id)
    .with_psychological_attribute("risk_aversion".to_string(), 0.7)
    .with_socioeconomic_attribute("income".to_string(), 75000.0);

let choice_module = SimpleChoiceModule::new();
let agent = ConsumerAgent::new(attributes, choice_module);
```

### Complete Model Setup
```rust
// Create environment with rules
let environment = Environment::new(interaction_rules);

// Create information processing
let mut transformer = Transformer::new(100.0);
transformer.add_filter(ReliabilityFilter::new(0.5));

// Create and configure model
let config = ModelConfiguration::new(
    "My Simulation".to_string(),
    "Consumer choice simulation".to_string()
);
let mut model = ConsumerChoiceModel::new(config, environment, transformer);

// Add agents and run
model.add_agent(agent)?;
model.run()?;
```

## Customization Examples

### Custom Choice Module
```rust
#[derive(Debug)]
struct MyChoiceModule {
    preferences: HashMap<String, f64>,
}

impl ChoiceModule for MyChoiceModule {
    type Choice = ProductChoice;
    type Context = MarketContext;
    
    fn make_choice(&self, choices: Vec<Self::Choice>, context: &Self::Context, trigger: TriggerType) -> Result<Option<Self::Choice>> {
        // Your custom decision logic here
        Ok(choices.into_iter().next())
    }
    
    // Implement other required methods...
}
```

### Custom Information Filter
```rust
#[derive(Debug)]
struct TopicFilter {
    allowed_topics: Vec<String>,
}

impl InformationFilter for TopicFilter {
    fn filter_information(&self, information: Vec<Information>, agent_id: &AgentId, context: &FilterContext) -> Result<Vec<Information>> {
        Ok(information.into_iter()
            .filter(|info| self.allowed_topics.iter().any(|topic| info.topic.contains(topic)))
            .collect())
    }
    
    // Implement other required methods...
}
```

## Advanced Features

### Async Simulation
```rust
#[cfg(feature = "async")]
{
    // Run async simulation
    model.run().await?;
    
    // Process agents concurrently
    for agent in agents {
        let choice = agent.process_trigger(trigger, choices, &context, time).await?;
    }
}
```

### Serialization Support
```rust
#[cfg(feature = "serde")]
{
    // Save model state
    let json = serde_json::to_string(&model.statistics())?;
    
    // Load agent attributes  
    let attributes: BasicAgentAttributes = serde_json::from_str(&data)?;
}
```

## Development Workflow

```bash
# Development cycle
cargo check          # Fast compilation check  
cargo clippy         # Linting and suggestions
cargo fmt            # Code formatting
cargo test           # Run all tests
cargo doc --open     # Generate documentation
cargo build --release  # Optimized build
```

## Documentation

Generate full documentation with:
```bash
cargo doc --open --all-features
```

This will create comprehensive API documentation with examples for all public types and traits.

## Contributing

The project structure supports easy extension:
- Add new choice modules by implementing `ChoiceModule`
- Create custom environments by implementing asset traits
- Add information processors via filter/distorter traits
- Extend validation with custom `ValidationRules`
