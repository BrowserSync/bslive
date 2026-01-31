use crate::task_trigger::TaskTrigger;
use bsnext_dto::internal::TaskResult;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "TaskResult")]
pub struct Invocation(pub u64, pub TaskTrigger);

impl Invocation {
    pub fn sqid(&self) -> String {
        let sqids = sqids::Sqids::default();
        sqids
            .encode(&[self.0])
            .unwrap_or_else(|_| self.0.to_string())
    }
}
