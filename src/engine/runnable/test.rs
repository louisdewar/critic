use crate::codegen::config::InputRef;

use super::{BasicRunnable, Runnable};

pub struct TestRunnable {
    pub should_panic: bool,
    pub subprocess: bool,
    pub basic_runnable: BasicRunnable,
}

// TODO: From<FixtureConfig>

impl Runnable for TestRunnable {
    fn run(&self, input: super::RunnableInput) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: panic handling, timeouts, etc...

        self.basic_runnable.run(input)
    }

    fn inputs(&self) -> &[InputRef] {
        self.basic_runnable.inputs()
    }
}
