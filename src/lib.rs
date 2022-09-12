pub use critic_sys::{fixture, test};
use engine::Engine;

mod engine;

pub(crate) mod codegen;

#[doc(hidden)]
pub mod __internal {
    pub use linkme;

    pub use crate::engine::fixture::Fixture;
    pub use crate::engine::test_definition::TestDefinition;

    pub use crate::engine::runnable::{
        BasicRunnable, FixtureRunnable, RunnableInput, TestRunnable,
    };

    pub use crate::codegen::config::{FixtureConfig, InputRef, TestConfig};
}

pub fn run_tests(
    tests: &[fn() -> __internal::TestConfig],
    fixtures: &[fn() -> __internal::FixtureConfig],
) {
    let mut engine = Engine::new(tests, fixtures);

    engine.run();
}

#[macro_export]
macro_rules! critic_test_main {
    () => {
        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        mod __critic_test_internals {
            #[$crate::__internal::linkme::distributed_slice]
            pub static CRITIC_INTERNAL_TESTS: [fn() -> critic::__internal::TestConfig] = [..];
            #[$crate::__internal::linkme::distributed_slice]
            pub static CRITIC_INTERNAL_FIXTURES: [fn() -> critic::__internal::FixtureConfig] = [..];
        }

        fn main() {
            $crate::run_tests(
                &self::__critic_test_internals::CRITIC_INTERNAL_TESTS,
                &self::__critic_test_internals::CRITIC_INTERNAL_FIXTURES,
            );
        }
    };
}
