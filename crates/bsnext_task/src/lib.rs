use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

pub mod as_actor;
pub mod invocation;
pub mod invocation_result;
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

impl Default for OverlappingOpts {
    fn default() -> Self {
        Self {
            max_concurrent_items: 5,
            exit_on_failure: true,
        }
    }
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

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Debug, Clone, Default)]
pub struct NodePath {
    inner: Vec<PathSegment>,
}

impl Display for NodePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.inner
                .iter()
                .map(|s| match s {
                    PathSegment::Content(c) => sqid_short(c.id()),
                    PathSegment::Index(index) => index.id().to_string(),
                })
                .collect::<Vec<String>>()
                .join(".")
        )
    }
}

impl NodePath {
    pub fn append(&mut self, segment: PathSegment) {
        self.inner.push(segment);
    }
    pub fn segments(&self) -> &[PathSegment] {
        &self.inner
    }
    pub fn path_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.inner.hash(&mut hasher);
        hasher.finish()
    }
    pub fn root_for(cid: ContentId) -> Self {
        let mut node_path = Self::default();
        node_path.append(PathSegment::Content(cid));
        node_path
    }
    pub fn as_string(&self) -> String {
        self.inner
            .iter()
            .map(|seg| seg.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Debug, Copy, Clone)]
pub enum PathSegment {
    Content(ContentId),
    Index(IndexId),
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            PathSegment::Content(c) => sqid_short(c.id()),
            PathSegment::Index(i) => sqid_short(i.id()),
        };
        write!(f, "{string}")
    }
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Copy, Clone)]
pub struct ContentId {
    inner: u64,
}

impl ContentId {
    pub fn new(id: u64) -> Self {
        Self { inner: id }
    }
}

impl ContentId {
    pub fn id(&self) -> u64 {
        self.inner
    }
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Copy, Clone)]
pub struct IndexId {
    inner: u64,
}

impl IndexId {
    pub fn new(id: u64) -> Self {
        Self { inner: id }
    }
}

impl IndexId {
    pub fn id(&self) -> u64 {
        self.inner
    }
}

pub fn sqid(id: u64) -> String {
    let sqids = sqids::Sqids::default();
    sqids.encode(&[id]).unwrap_or_else(|_| id.to_string())
}

pub fn sqid_short(id: u64) -> String {
    let sqids = sqids::Sqids::default();
    let sqid = sqids.encode(&[id]).unwrap();
    sqid.get(0..6).unwrap_or(&sqid).to_string()
}
