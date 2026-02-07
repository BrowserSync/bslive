pub mod as_actor;
pub mod invocation;
pub mod task_entry;
pub mod task_report;
pub mod task_scope;
pub mod task_scope_runner;
pub mod task_trigger;

/// The `RunKind` enum represents the type of execution or arrangement of a set of operations or elements.
/// It provides two distinct variants: `Sequence` and `Overlapping`.
///
/// ## Variants
///
/// - `Sequence`:
///   Represents a straightforward sequential arrangement or execution.
///   Operations or elements will proceed one after another in the specified order.
///
/// - `Overlapping`:
///   Represents an overlapping arrangement where operations or elements can overlap or run concurrently,
///   based on specific options provided.
///
///   - `opts`: A field of type `OverlappingOpts` that contains the configuration or parameters
///     dictating the behavior of overlapping operations.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum RunKind {
    Sequence { opts: SequenceOpts },
    Overlapping { opts: OverlappingOpts },
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct OverlappingOpts {
    pub max_concurrent_items: u8,
    pub exit_on_failure: bool,
}

impl OverlappingOpts {
    pub fn new(max_concurrent_items: u8, exit_on_failure: bool) -> Self {
        Self {
            max_concurrent_items,
            exit_on_failure,
        }
    }
}
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct SequenceOpts {
    pub exit_on_failure: bool,
}

impl Default for SequenceOpts {
    fn default() -> Self {
        Self {
            exit_on_failure: true,
        }
    }
}

impl SequenceOpts {
    pub fn new(exit_on_failure: bool) -> Self {
        Self { exit_on_failure }
    }
}
