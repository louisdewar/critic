use std::any::TypeId;

use crate::engine::runnable::RunnableFn;

#[derive(Clone, Debug, Copy)]
// TODO: I think only T = TypeId is ever used so maybe get rid of generics
pub enum InputRef<T = TypeId> {
    Shared(T),
    Exclusive(T),
    Owned(T),
}

impl<T> InputRef<T> {
    pub fn shared(inner: T) -> Self {
        InputRef::Shared(inner)
    }

    pub fn exclusive(inner: T) -> Self {
        InputRef::Exclusive(inner)
    }

    pub fn owned(inner: T) -> Self {
        InputRef::Owned(inner)
    }

    pub fn inner(&self) -> &T {
        use InputRef::*;
        match &self {
            Shared(inner) | Exclusive(inner) | Owned(inner) => inner,
        }
    }
}

impl InputRef<TypeId> {
    pub fn id(&self) -> TypeId {
        *self.inner()
    }
}

/// All the configuration for a test
pub struct TestConfig {
    /// The test is expected to panic
    pub should_panic: bool,
    /// The test should run in a subprocess
    pub subprocess: bool,
    /// The inputs (fixtures) that this test should receive
    pub inputs: Vec<InputRef>,
    /// The optional name of a group of runnables that cannot be run in parallel with each other
    pub exclusion_group: Option<String>,
    /// The runnable function
    pub runnable_fn: RunnableFn,
    /// The full path of the module the test is in
    pub module_path: String,
    /// The name of the test (when combined with module_path it must be globally unique).
    pub name: String,
}

/// All the configuration for a fixture
pub struct FixtureConfig {
    /// The inputs (fixtures) that this fixture should receive
    /// NOTE: THIS IS NOT CURRENTLY SUPPORTED
    pub inputs: Vec<InputRef>,
    /// The type of the output of this fixture
    pub output: TypeId,
    /// The runnable function
    pub runnable_fn: RunnableFn,
    /// The full path of the module the producer function is in
    pub module_path: String,
    /// The name of the producer function
    pub name: String,
}
