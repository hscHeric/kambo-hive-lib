use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{result::TaskResult, task::Task};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    RequestTask { worker_id: Uuid },
    ReportResult { worker_id: Uuid, result: TaskResult },
    Heartbeat { worker_id: Uuid },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    AssignTask {
        task: Task,
    },
    NoTaskAvailable,
    Ack,
    Command {
        command_type: String,
        payload: String,
    },
}
