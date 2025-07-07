use uuid::Uuid;

use super::{result::TaskResult, task::Task};

pub trait GARunner: Send + Sync + 'static {
    fn run(&self, task: Task, worker_id: Uuid) -> TaskResult;
}
