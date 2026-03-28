use bsnext_task::invocation::SpecId;
use bsnext_task::invocation_result::{InvocationConclusion, InvocationResult};
use bsnext_task::task_report::{TaskOk, TaskReport};

#[test]
fn add() {
    let task_report = TaskReport {
        result: InvocationResult {
            conclusion: InvocationConclusion::Ok(TaskOk),
            node_path: SpecId::new(0),
            task_reports: vec![],
        },
        spec_id: SpecId::new(0),
    };
    let invocation_result = InvocationResult {
        conclusion: InvocationConclusion::Ok(TaskOk),
        task_reports: vec![task_report],
        node_path: SpecId::new(0),
    };
    assert!(invocation_result.is_ok());
}
