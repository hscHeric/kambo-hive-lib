use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub graph_id: String,
    pub run_number: u32,
    pub ag_config: String,
}

impl Task {
    pub fn new(graph_id: String, run_number: u32, ag_config: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            graph_id,
            run_number,
            ag_config,
        }
    }
}
