use bsnext_task::invocation_result::{InvocationConclusion, InvocationResult};
use bsnext_task::task_report::{TaskOk, TaskReport};
use bsnext_task::{ContentId, NodePath};

#[test]
fn add() {
    let path = NodePath::root_for(ContentId::new(0));
    let task_report = TaskReport {
        result: InvocationResult {
            conclusion: InvocationConclusion::Ok(TaskOk),
            node_path: path.clone(),
            task_reports: vec![],
        },
        node_path: path.clone(),
    };
    let invocation_result = InvocationResult {
        conclusion: InvocationConclusion::Ok(TaskOk),
        task_reports: vec![task_report],
        node_path: path,
    };
    assert!(invocation_result.is_ok());
}
