mod schedule;

use self::schedule::START_NODE;

use super::{
    dependencies::Dependencies,
    runnable::{Runnable, RunnableInput},
    EngineConfig, Label,
};
use crate::{
    codegen::config::InputRef,
    engine::runnable::{BasicRunnable, FixtureRunnable, TestRunnable},
};
use parking_lot::RwLock;
pub use schedule::Schedule;
use schedule::ScheduleBuilder;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::Deref,
};
use uuid::Uuid;

pub struct Runner {
    schedule: Schedule,
    runnables: HashMap<Uuid, Box<dyn Runnable>>,
    // TODO: think of better name and then define trait instead of using Any and make sure it has
    // Send
    outputs: HashMap<TypeId, RwLock<Box<dyn Any>>>,
}

// TODO: eventually once tests can also output data, try to abstract away what is a test and what
// isn't to a higher level, then make this just receive "nodes" using the label system to figure
// out interdependency maybe (also higher level should remove fixtures that aren't used anywhere).
impl Runner {
    pub fn new(config: &EngineConfig) -> Runner {
        let mut builder = ScheduleBuilder::new();
        let mut runnables = HashMap::new();
        let mut fixture_nodes: HashMap<TypeId, Uuid> = HashMap::new();

        for (test_id, test_config) in &config.tests {
            builder.register_node(*test_id);
            if let Some(excludes_with) = &test_config.exclusion_group {
                for other in config
                    .labels
                    .get(&Label::User(excludes_with.to_string()))
                    .unwrap()
                {
                    if test_id != other {
                        builder.add_exclusion(*test_id, *other);
                    }
                }
            }

            runnables.insert(
                *test_id,
                Box::new(TestRunnable {
                    should_panic: test_config.should_panic,
                    subprocess: test_config.subprocess,
                    basic_runnable: BasicRunnable {
                        inputs: test_config.inputs.clone(),
                        runner: test_config.runnable_fn,
                    },
                }) as Box<dyn Runnable>,
            );

            for input in &test_config.inputs {
                use crate::codegen::config::InputRef::*;

                let input_uuid = *fixture_nodes.entry(input.id()).or_insert_with(|| {
                    let input_uuid = Uuid::new_v4();
                    let fixture_config = config
                        .fixtures
                        .get(&dbg!(input.id()))
                        .expect("input is not a fixture");
                    runnables.insert(
                        input_uuid,
                        Box::new(FixtureRunnable {
                            runnable: BasicRunnable {
                                inputs: vec![],
                                runner: fixture_config.runnable_fn,
                            },
                        }),
                    );
                    builder.register_node(*test_id);

                    input_uuid
                });

                match *input {
                    Shared(_) => {
                        builder.add_dependency(input_uuid, *test_id);
                    }
                    Exclusive(_) => {
                        builder.add_dependency(input_uuid, *test_id);
                        for other in config.labels.get(&Label::Dependency(input.id())).unwrap() {
                            if other != &input_uuid {
                                builder.add_exclusion(*other, input_uuid);
                            }
                        }
                    }
                    Owned(_) => {}
                }
            }
        }

        Runner {
            schedule: builder.build(),
            runnables,
            outputs: Default::default(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.schedule.next() {
                schedule::NextInSchedule::Running => unreachable!("currently single threaded"),
                schedule::NextInSchedule::Completed => return,
                schedule::NextInSchedule::Next(id) => {
                    if let Some(runnable) = self.runnables.get(&id) {
                        // let mut shared_guards = Vec::new();
                        // let mut exclusive_guards = Vec::new();

                        // TODO: sub node tainting...
                        let mut dependencies = Dependencies::new();
                        for input in runnable.inputs() {
                            use InputRef::*;
                            match input {
                                Shared(id) => {
                                    let guard = self
                                        .outputs
                                        .get(id)
                                        .unwrap()
                                        .try_read()
                                        .expect("mutual exclusion prevents locks");
                                    // shared_guards.push(guard);
                                    dependencies.add_shared(*id, guard);
                                }
                                Exclusive(id) => {
                                    let guard = self
                                        .outputs
                                        .get(id)
                                        .unwrap()
                                        .try_write()
                                        .expect("mutual exclusion prevents locks");
                                    // exclusive_guards.push(guard);
                                    dependencies.add_exclusive(*id, guard);
                                }
                                _ => todo!(),
                            }
                        }

                        let mut receiver = Default::default();

                        let input = RunnableInput {
                            dependencies,
                            receiver: &mut receiver,
                        };
                        runnable.run(input).expect("test failed");

                        self.outputs.extend(
                            receiver
                                .outputs
                                .into_iter()
                                .map(|(key, value)| (key, RwLock::new(value))),
                        );
                        self.schedule.complete_node(id);
                    } else if id == START_NODE {
                        self.schedule.complete_node(id);
                        continue;
                    } else {
                        panic!("unknown ID: {}", id);
                    }
                }
            }
        }
    }
}
