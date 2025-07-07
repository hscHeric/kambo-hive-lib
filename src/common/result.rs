use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub graph_id: String,
    pub worker_id: Uuid,
    pub fitness: f64,
    pub solution_data: Vec<u8>,
    pub interations_run: u32,
    pub processing_time_ms: u64,
}
