use crate::codegen::config::InputRef;

use super::{BasicRunnable, Runnable};

pub struct FixtureRunnable {
    pub runnable: BasicRunnable,
}

// TODO: From<FixtureConfig>

impl Runnable for FixtureRunnable {
    fn run(&self, input: super::RunnableInput) -> Result<(), Box<dyn std::error::Error>> {
        self.runnable.run(input)
    }

    fn inputs(&self) -> &[InputRef] {
        self.runnable.inputs()
    }
}
