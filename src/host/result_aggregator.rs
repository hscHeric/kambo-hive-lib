use log::info;
use serde::Serialize;
use std::{collections::HashMap, error::Error, fs};
use uuid::Uuid;

use super::task_manager::{TaskManager, TaskStatus};
use crate::common::TaskResult;

#[derive(Serialize)]
struct ReportGraphDetails {
    results_collected: usize,
    best_fitness: f64,
    avg_processing_time_ms: f64,
    total_processing_time_ms: u64,
    results: Vec<TaskResult>,
}

#[derive(Serialize)]
struct ReportStatusSummary {
    total: usize,
    completed: usize,
    failed: usize,
    pending: usize,
    assigned: usize,
}

#[derive(Serialize, Clone)]
struct WorkerReport {
    worker_id: Uuid,
    tasks_completed: u32,
    total_processing_time_ms: u64,
    avg_processing_time_ms: f64,
}

#[derive(Serialize)]
struct JsonReport {
    task_summary: ReportStatusSummary,
    graphs: HashMap<String, ReportGraphDetails>,
    workers: Vec<WorkerReport>, // Novo campo para estatísticas dos workers
}

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

    pub fn generate_and_save_report(
        &self,
        task_manager: &TaskManager,
        file_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        info!("Gerando relatório final para {}", file_path);

        let task_summary = ReportStatusSummary {
            total: task_manager.get_total_tasks(),
            completed: task_manager.get_completed_tasks_count(),
            failed: task_manager
                .get_tasks_status()
                .values()
                .filter(|&&s| s == TaskStatus::Failed)
                .count(),
            pending: task_manager
                .get_tasks_status()
                .values()
                .filter(|&&s| s == TaskStatus::Pending)
                .count(),
            assigned: task_manager
                .get_tasks_status()
                .values()
                .filter(|&&s| s == TaskStatus::Assigned)
                .count(),
        };

        let graphs: HashMap<String, ReportGraphDetails> = self
            .get_all_results()
            .iter()
            .map(|(graph_id, results)| {
                let total_time_ms: u64 = results.iter().map(|r| r.processing_time_ms).sum();
                let avg_time_ms = if results.is_empty() {
                    0.0
                } else {
                    total_time_ms as f64 / results.len() as f64
                };
                let best_fitness = results
                    .iter()
                    .map(|r| r.fitness)
                    .fold(f64::NEG_INFINITY, f64::max);

                (
                    graph_id.clone(),
                    ReportGraphDetails {
                        results_collected: results.len(),
                        best_fitness,
                        avg_processing_time_ms: avg_time_ms,
                        total_processing_time_ms: total_time_ms,
                        results: results.clone(),
                    },
                )
            })
            .collect();

        let mut worker_stats: HashMap<Uuid, (u32, u64)> = HashMap::new();
        for results in self.get_all_results().values() {
            for result in results {
                let stats = worker_stats.entry(result.worker_id).or_insert((0, 0));
                stats.0 += 1;
                stats.1 += result.processing_time_ms;
            }
        }

        let workers: Vec<WorkerReport> = worker_stats
            .into_iter()
            .map(|(worker_id, (tasks_completed, total_processing_time_ms))| {
                let avg_processing_time_ms = if tasks_completed > 0 {
                    total_processing_time_ms as f64 / tasks_completed as f64
                } else {
                    0.0
                };
                WorkerReport {
                    worker_id,
                    tasks_completed,
                    total_processing_time_ms,
                    avg_processing_time_ms,
                }
            })
            .collect();

        let report = JsonReport {
            task_summary,
            graphs,
            workers,
        };

        let json_data = serde_json::to_string_pretty(&report)?;
        fs::write(file_path, json_data)?;

        info!("Relatório salvo com sucesso em '{}'", file_path);

        Ok(())
    }
}

impl Default for ResultAggregator {
    fn default() -> Self {
        Self::new()
    }
}
