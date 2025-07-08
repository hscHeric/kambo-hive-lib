use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use log::{error, info};
use serde::Serialize;
use uuid::Uuid;

use super::result_aggregator::ResultAggregator;

#[derive(Serialize, Clone)]
struct SaverTaskResult {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub fitness: f64,
    pub solution_data: Vec<u8>,
    pub interations_run: u32,
    pub processing_time_ms: u64,
}

#[derive(Serialize)]
struct SaverResults {
    name: String,
    results: Vec<SaverTaskResult>,
}

pub fn start(aggregator: Arc<Mutex<ResultAggregator>>, file_path: String, interval_secs: u64) {
    info!("Salvamento periódico ativado. Arquivo: '{file_path}', Intervalo: {interval_secs}s.");

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;
            let results_guard = aggregator.lock().await;

            if results_guard.get_results_collected() == 0 {
                info!("Nenhum resultado para salvar, pulando ciclo de salvamento.");
                continue;
            }

            info!(
                "Preparando {} resultados para salvar em '{}'...",
                results_guard.get_results_collected(),
                file_path
            );

            let formatted_results: Vec<SaverResults> = results_guard
                .get_all_results()
                .iter()
                .map(|(graph_name, task_results)| SaverResults {
                    name: graph_name.clone(),
                    results: task_results
                        .iter()
                        .map(|tr| SaverTaskResult {
                            task_id: tr.task_id,
                            worker_id: tr.worker_id,
                            fitness: tr.fitness,
                            solution_data: tr.solution_data.clone(),
                            interations_run: tr.interations_run,
                            processing_time_ms: tr.processing_time_ms,
                        })
                        .collect(),
                })
                .collect();

            match serde_json::to_string_pretty(&formatted_results) {
                Ok(json_data) => {
                    if let Err(e) = fs::write(&file_path, json_data) {
                        error!("Falha ao escrever no arquivo de resultados '{file_path}': {e}");
                    } else {
                        info!("Resultados salvos com sucesso.");
                    }
                }
                Err(e) => {
                    error!("Falha ao serializar resultados para JSON: {e}");
                }
            }
        }
    });
}
