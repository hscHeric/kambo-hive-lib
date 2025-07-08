use log::{debug, error, info};
use serde_json;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;
use uuid::Uuid;

use crate::common::{GARunner, Request, Response};

pub async fn start_worker<T: GARunner>(
    host_addr: &str,
    worker_id: Uuid,
    ga_runner: Arc<T>,
) -> Result<(), Box<dyn Error>> {
    info!("Trabalhador {worker_id} tentando se conectar ao host em {host_addr}");

    loop {
        match TcpStream::connect(host_addr).await {
            Ok(stream) => {
                info!("Trabalhador {worker_id} conectado ao host.");
                if let Err(e) =
                    handle_host_connection(stream, worker_id, Arc::clone(&ga_runner)).await
                {
                    error!("Conexão com o host perdida ou erro: {e}");
                }
                info!("Tentando reconectar em 5 segundos...");
                sleep(Duration::from_secs(5)).await;
            }
            Err(e) => {
                error!("Falha ao conectar ao host: {e}. Tentando novamente em 5 segundos...");
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_host_connection<T: GARunner>(
    stream: TcpStream,
    worker_id: Uuid,
    ga_runner: Arc<T>,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();

        let request = Request::RequestTask { worker_id };
        let encoded_request = serde_json::to_vec(&request)?;
        reader.write_all(&encoded_request).await?;
        reader.write_all(b"\n").await?;
        reader.flush().await?;
        debug!("Trabalhador {worker_id} solicitou uma tarefa.");

        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Err("Host desconectado.".into());
        }

        let response: Response = serde_json::from_str(&line)?;
        debug!("Trabalhador {worker_id} recebeu a resposta: {response:?}");

        match response {
            Response::AssignTask { task } => {
                info!("Trabalhador {} recebeu a tarefa {}", worker_id, task.id);
                let result = ga_runner.run(task, worker_id);
                info!(
                    "Trabalhador {} terminou a tarefa {}. Melhor fitness: {}",
                    worker_id, result.task_id, result.fitness
                );

                let clone_result = result.clone();
                let report_request = Request::ReportResult { worker_id, result };
                let encoded_report = serde_json::to_vec(&report_request)?;
                reader.write_all(&encoded_report).await?;
                reader.write_all(b"\n").await?;
                reader.flush().await?;
                debug!(
                    "Trabalhador {} reportou o resultado da tarefa {}",
                    worker_id, clone_result.task_id
                );
            }
            Response::NoTaskAvailable => {
                info!(
                    "Trabalhador {worker_id} recebeu NoTaskAvailable. Aguardando novas tarefas..."
                );
                sleep(Duration::from_secs(2)).await;
            }
            Response::Ack => {
                debug!("Trabalhador {worker_id} recebeu Ack.");
            }
            Response::Command {
                command_type,
                payload,
            } => {
                info!(
                    "Trabalhador {worker_id} recebeu comando: {command_type} com payload {payload}"
                );
            }
        }
    }
}
