use crate::input_src::{FileInputSrc, InputSrc};
use crate::route::WatchSpec;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct InputWatchable {
    pub dirs: Vec<PathBuf>,
    pub watch_spec: WatchSpec,
}

pub fn to_input_watchable(src_list: &[InputSrc]) -> Vec<InputWatchable> {
    src_list
        .iter()
        .map(|input_src| match input_src {
            InputSrc::File(FileInputSrc { absolute }) => InputWatchable {
                dirs: vec![absolute.to_owned()],
                watch_spec: WatchSpec::default(),
            },
        })
        .collect()
}
