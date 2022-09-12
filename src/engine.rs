use std::{any::TypeId, collections::HashMap};

use uuid::Uuid;

use crate::codegen::config::{FixtureConfig, InputRef, TestConfig};

use self::runner::Runner;

pub mod dependencies;
pub mod fixture;
pub mod runnable;
pub mod runner;
pub mod test_definition;

pub struct EngineConfig {
    pub tests: HashMap<Uuid, TestConfig>,
    pub fixtures: HashMap<TypeId, FixtureConfig>,
    pub groups: HashMap<String, TestGroup>,
    pub labels: HashMap<Label, Vec<Uuid>>,
    // pub fixture_nodes: HashMap<TypeId, Uuid>,
}

pub struct Engine {
    runner: Runner,
}

pub struct LifeCycle {
    // TODO: maybe in own file?
    // before_each: Option<...>
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Label {
    /// A label that is the user namespace
    User(String),
    /// A label for tracking dependencies
    /// Typically used to indicate that a node takes an input with this TypeID
    Dependency(TypeId),
}

pub struct TestGroup {
    name: String,
    lifecycle: LifeCycle,
    tests: Vec<Uuid>,
}

impl TestGroup {
    fn new(name: String) -> Self {
        TestGroup {
            name,
            lifecycle: LifeCycle {},
            tests: Vec::new(),
        }
    }
}

impl Engine {
    pub(crate) fn new(tests: &[fn() -> TestConfig], fixtures: &[fn() -> FixtureConfig]) -> Self {
        let fixtures: HashMap<_, _> = fixtures
            .iter()
            .map(|definer| definer())
            .map(|config| (dbg!(config.output), config))
            .collect();

        let tests: HashMap<_, _> = tests
            .iter()
            .map(|definer| definer())
            .map(|config| (Uuid::new_v4(), config))
            .collect();

        let mut groups = HashMap::new();
        let mut labels: HashMap<Label, Vec<Uuid>> = HashMap::new();
        // let mut fixture_nodes = HashMap::new();

        for (test_id, config) in tests.iter() {
            groups
                .entry(config.module_path.clone())
                .or_insert_with(|| TestGroup::new(config.module_path.clone()))
                .tests
                .push(*test_id);

            if let Some(exclusion_label) = &config.exclusion_group {
                labels
                    .entry(Label::User(exclusion_label.clone()))
                    .or_default()
                    .push(*test_id);
            }

            for input in &config.inputs {
                use InputRef::*;
                match input {
                    Shared(id) | Exclusive(id) => {
                        labels
                            .entry(Label::Dependency(*id))
                            .or_default()
                            .push(*test_id);
                    }
                    Owned(_) => {}
                }
            }
        }

        let config = EngineConfig {
            tests,
            fixtures,
            groups,
            labels,
            // fixture_nodes,
        };
        let runner = Runner::new(&config);

        Engine { runner }
    }

    pub fn run(&mut self) {
        self.runner.run();
    }
}
