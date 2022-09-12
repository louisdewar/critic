use super::runnable::TestRunnable;

/// Represents a runnable test and its configuration.
/// This should not be contructed manually but through `#[critic::test]` attributes.
pub struct TestDefinition {
    pub module: String,
    pub name: String,
    pub runnable: TestRunnable,
}

impl TestDefinition {
    fn fqn(&self) -> String {
        format!("{}:{}", self.module, self.name)
    }
}
