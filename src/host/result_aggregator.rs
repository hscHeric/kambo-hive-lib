use std::{collections::HashMap, error::Error};

use log::info;

use crate::common::TaskResult;

pub struct ResultAggregator {
    results_by_graph: HashMap<String, Vec<TaskResult>>,
    total_results_collected: usize,
}

impl ResultAggregator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            results_by_graph: HashMap::new(),
            total_results_collected: 0,
        }
    }

    pub fn add_result(&mut self, result: TaskResult) -> Result<(), Box<dyn Error>> {
        let graph_id = result.graph_id.clone();
        self.results_by_graph
            .entry(graph_id)
            .or_default()
            .push(result);

        self.total_results_collected += 1;
        info!(
            "Resultado adicionado. total de resultados: {}",
            self.total_results_collected
        );

        Ok(())
    }

    #[must_use]
    pub const fn get_results_collected(&self) -> usize {
        self.total_results_collected
    }

    #[must_use]
    pub const fn get_all_results(&self) -> &HashMap<String, Vec<TaskResult>> {
        &self.results_by_graph
    }
}

impl Default for ResultAggregator {
    fn default() -> Self {
        Self::new()
    }
}
