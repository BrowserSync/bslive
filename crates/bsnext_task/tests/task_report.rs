use bsnext_task::invocation::InvocationId;
use bsnext_task::invocation_result::{InvocationConclusion, InvocationResult};
use bsnext_task::task_report::{TaskOk, TaskReport};

#[test]
fn add() {
    assert_eq!(1 + 1, 2);
    let task_report = TaskReport {
        result: InvocationResult {
            conclusion: InvocationConclusion::Ok(TaskOk),
            invocation_id: InvocationId(0),
            task_reports: vec![],
        },
        id: 0,
    };
    let invocation_result = InvocationResult {
        conclusion: InvocationConclusion::Ok(TaskOk),
        task_reports: vec![task_report],
        invocation_id: InvocationId(0),
    };
    assert!(invocation_result.is_ok());
}
