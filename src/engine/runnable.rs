use std::{
    any::{Any, TypeId},
    collections::HashMap,
    error::Error,
};

use crate::codegen::config::InputRef;

use super::dependencies::Dependencies;

mod fixture;
mod lifecycle;
mod test;

pub use fixture::FixtureRunnable;
pub use test::TestRunnable;

pub type RunnableFn = &'static dyn Fn(RunnableInput) -> Result<(), Box<dyn Error>>;

/// Receives output from a runnable
#[derive(Default)]
pub struct Receiver {
    pub(crate) outputs: HashMap<TypeId, Box<dyn Any>>,
    // TODO: expecting: HashSet<TypeId> ? that way assert!(expecting.is_empty()) at end
}

impl Receiver {
    pub fn receive_output<T: 'static>(&mut self, output: T) {
        self.outputs
            .insert(TypeId::of::<T>(), Box::new(output) as Box<dyn Any>);
    }
}

/// The struct given to a runnable
pub struct RunnableInput<'dep, 'recv> {
    pub dependencies: Dependencies<'dep>,
    pub receiver: &'recv mut Receiver,
}

pub struct BasicRunnable {
    pub inputs: Vec<InputRef>,
    pub runner: RunnableFn,
}

/// Describes a runnable test, fixture or lifecycle without strong reference to execution order.
pub trait Runnable {
    fn run(&self, input: RunnableInput<'_, '_>) -> Result<(), Box<dyn Error>>;
    fn inputs(&self) -> &[InputRef];
}

impl Runnable for BasicRunnable {
    fn run(&self, input: RunnableInput<'_, '_>) -> Result<(), Box<dyn Error>> {
        (self.runner)(input)
    }

    fn inputs(&self) -> &[InputRef] {
        &self.inputs
    }
}
