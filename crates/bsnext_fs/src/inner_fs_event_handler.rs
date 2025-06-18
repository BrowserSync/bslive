use crate::FsEvent;
use std::path::PathBuf;

#[derive(actix::Message, Hash, PartialEq, Eq, Ord, PartialOrd, Debug, Clone)]
#[rtype(result = "()")]
pub(crate) struct InnerChangeEvent {
    pub absolute_path: PathBuf,
}

#[derive(actix::Message, Hash, PartialEq, Eq, Ord, PartialOrd, Debug, Clone)]
#[rtype(result = "()")]
pub struct MultipleInnerChangeEvent {
    pub events: Vec<FsEvent>,
}

#[derive(actix::Message, Hash, PartialEq, Eq, Ord, PartialOrd, Debug, Clone)]
#[rtype(result = "()")]
pub struct SingleInnerChangeEvent {
    pub event: FsEvent,
}
