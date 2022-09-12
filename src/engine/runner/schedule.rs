use std::collections::{HashMap, HashSet, VecDeque};

use uuid::Uuid;

/// The UUID of the starting node in the dependency graph (the node that has no dependencies and
/// every other node is a transitive dependency of this node.
///
/// The direct children of this node all have zero actual dependencies.
pub const START_NODE: Uuid = uuid::uuid!("f0993081-13f5-45c9-9fe7-5b1de19b20ee");

/// Keeps track of which nodes should run when taking into account dependencies, mutual exlcusion
/// and more.
/// The schedule is not aware what type nodes are, it only knows Uuids.
pub struct Schedule {
    nodes: HashMap<Uuid, Node>,
    /// A list of nodes that are unblocked but not active (i.e. they are excluded)
    waiting: HashSet<Uuid>,
    /// A mapping from a "running" node to a list of nodes it is currently blocking (i.e. nodes
    /// that are not waiting on dependencies but mutually exclude with that node).
    /// The blocked nodes may be mutually excluding with more than one running node.
    excluding: HashMap<Uuid, Vec<Uuid>>,
    /// Nodes that have been added to the queue or yielded but not yet completed.
    active: HashSet<Uuid>,
    /// The queue of nodes that are able to run (no dependency/exclusion).
    /// Any nodes is here count as part of mutual exclusion rules of running nodes.
    queue: VecDeque<Uuid>,
}

#[derive(Clone, Debug)]
pub enum NextInSchedule {
    /// An item is ready to run.
    Next(Uuid),
    /// The schedule has completed all nodes.
    Completed,
    /// Some nodes are running but none are ready to start (either because of dependency/exclusion
    /// or the very last nodes are finishing off).
    Running,
}

impl Schedule {
    // Finds the next node that can run or None if all nodes have been completed or are currently
    // running.
    pub fn next(&mut self) -> NextInSchedule {
        if let Some(item) = self.queue.pop_front() {
            debug_assert!(self.active.contains(&item), "node in queue was not active");
            return NextInSchedule::Next(item);
        }

        if self.nodes.is_empty() && self.excluding.is_empty() {
            return NextInSchedule::Completed;
        }

        assert!(!self.active.is_empty(), "deadlock found in scheduler");

        NextInSchedule::Running
    }

    /// Add the node to the queue (and to active).
    /// This does not check any pre-requisites (unblocked and non-excluded).
    fn add_to_queue(&mut self, node: Uuid) {
        if cfg!(debug_assertions) {
            let node = self.nodes.get(&node).unwrap();

            for excluded in &node.mutually_excludes {
                assert!(!self.active.contains(excluded));
            }
        }

        self.queue.push_back(node);
        assert!(self.active.insert(node));
    }

    /// TODO: look at where this is used and then rename and simplify
    /// Determines whether a node is able to be queued (neither blocked nor excluded).
    /// This will not check whether the node is already queued (this is a logical error).
    fn node_can_be_queued(&self, node: &Node) -> bool {
        if node.dependency_count > 0 {
            // Blocked
            return false;
        }

        for excluder in &node.mutually_excludes {
            if self.active.contains(excluder) {
                // Excluded
                return false;
            }
        }

        true
    }

    /// Mark a node as complete, and propogate to all dependents finding all nodes that could
    /// become active.
    /// This will also remove this node from the graph.
    pub fn complete_node(&mut self, node_id: Uuid) {
        let node = self
            .nodes
            .remove(&node_id)
            .expect("completed node that was unknown");
        assert!(
            self.active.remove(&node_id),
            "completed node that was not running"
        );

        assert_eq!(node.dependency_count, 0);

        if let Some(excluding) = self.excluding.remove(&node_id) {
            for excluded_id in &excluding {
                let excluded = self
                    .nodes
                    .get(excluded_id)
                    .expect("excluded nodes can't run, so it must still be in self.nodes");
                if self.node_can_be_queued(excluded) {
                    // new_excluding is the list of waiting nodes that the *about to be run*
                    // node is going to exclude
                    let new_excluding = excluded
                        .mutually_excludes
                        .iter()
                        .copied()
                        .filter(|item| self.waiting.contains(item))
                        .collect();
                    assert!(self.excluding.insert(*excluded_id, new_excluding).is_none());
                    assert!(self.waiting.remove(excluded_id));
                    self.add_to_queue(*excluded_id);
                }
            }
        }

        for dependent_id in node.dependents {
            let dependent = self
                .nodes
                .get_mut(&dependent_id)
                .expect("dependent must still exist");
            dependent.dependency_count -= 1;
            if dependent.dependency_count == 0 {
                let mut excluded = false;
                for excluder in &dependent.mutually_excludes {
                    if self.active.contains(excluder) {
                        excluded = true;
                        self.excluding
                            .entry(*excluder)
                            .or_default()
                            .push(dependent_id);
                    }
                }

                if excluded {
                    assert!(self.waiting.insert(dependent_id));
                } else {
                    self.add_to_queue(dependent_id);
                }
            }
        }
    }
}

/// Registers dependencies and other constraints to build a schedule
pub struct ScheduleBuilder {
    nodes: HashMap<Uuid, Node>,
    no_dependencies: HashSet<Uuid>,
}

struct Node {
    /// A list of the nodes that depend on this node
    dependents: HashSet<Uuid>,
    /// A list of the nodes that cannot be run in parallel with this node
    mutually_excludes: HashSet<Uuid>,
    /// The number of direct dependencies this Node has
    dependency_count: usize,
}

impl Node {
    pub fn new() -> Self {
        Node {
            dependents: HashSet::new(),
            mutually_excludes: HashSet::new(),
            dependency_count: 0,
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::new()
    }
}

impl ScheduleBuilder {
    pub fn new() -> Self {
        ScheduleBuilder {
            nodes: HashMap::new(),
            no_dependencies: HashSet::new(),
        }
    }

    pub fn build(mut self) -> Schedule {
        for node_id in &self.no_dependencies {
            // Dependent only on the start node.
            self.nodes.get_mut(node_id).unwrap().dependency_count = 1;
        }

        assert!(self
            .nodes
            .insert(
                START_NODE,
                Node {
                    dependents: self.no_dependencies.iter().copied().collect(),
                    mutually_excludes: Default::default(),
                    dependency_count: 0,
                },
            )
            .is_none());

        let mut schedule = Schedule {
            nodes: self.nodes,
            active: Default::default(),
            queue: Default::default(),
            waiting: Default::default(),
            excluding: Default::default(),
        };

        schedule.add_to_queue(START_NODE);

        schedule
    }

    fn entry(&mut self, id: Uuid) -> &mut Node {
        self.nodes.entry(id).or_insert_with(|| {
            self.no_dependencies.insert(id);
            Default::default()
        })
    }

    /// Ensures the node with the specified node is registered (either does nothing or inserts new)
    pub fn register_node(&mut self, id: Uuid) {
        self.entry(id);
    }

    /// Add dependency (child depends on parent)
    pub fn add_dependency(&mut self, parent: Uuid, child: Uuid) {
        let child_node = self.entry(child);
        child_node.dependency_count += 1;
        // Remove child from no_dependencies if it was in there
        if child_node.dependency_count == 1 {
            self.no_dependencies.remove(&child);
        }

        self.entry(parent).dependents.insert(child);
    }

    /// Marks two nodes as mutually exclusive
    pub fn add_exclusion(&mut self, a: Uuid, b: Uuid) {
        assert_ne!(a, b, "node can't mutually exclude itself");
        self.entry(a).mutually_excludes.insert(b);
        self.entry(b).mutually_excludes.insert(a);
    }
}
