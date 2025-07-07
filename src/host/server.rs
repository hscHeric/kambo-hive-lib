use log::{debug, error, info};
use std::error::Error;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::common::Request;
use crate::common::Response;
use crate::host::result_aggregator::ResultAggregator;
use crate::host::task_manager::TaskManager;

pub async fn start_server(
    addr: &str,
    task_manager: Arc<Mutex<TaskManager>>,
    result_aggregator: Arc<Mutex<ResultAggregator>>,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    info!("Host escutando em {addr}");

    loop {
        let (socket, remote_addr) = listener.accept().await?;
        info!("Worker {remote_addr}, se conectando");

        let task_manager_clone = Arc::clone(&task_manager);
        let result_aggregator_clone = Arc::clone(&result_aggregator);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, task_manager_clone, result_aggregator_clone).await
            {
                error!("Error {remote_addr}: {e}");
            }
        });
    }
}

async fn handle_client(
    socket: TcpStream,
    task_manager: Arc<Mutex<TaskManager>>,
    result_aggregator: Arc<Mutex<ResultAggregator>>,
) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(socket);
    let mut line_buffer = String::new();

    loop {
        line_buffer.clear(); // Limpa o buffer para a próxima linha

        // Lê até o delimitador de newline ('\n')
        let bytes_read = reader.read_line(&mut line_buffer).await?;

        if bytes_read == 0 {
            info!("Cliente desconectado.");
            return Ok(());
        }

        // Tenta desserializar a Request da linha lida
        let msg: Request = serde_json::from_str(&line_buffer)?;
        debug!(r"Recebida solicitação do trabalhador: {msg:?}");

        let response = match msg {
            Request::RequestTask { worker_id } => {
                let mut tm = task_manager.lock().await;
                if let Some(task) = tm.get_next_task(worker_id) {
                    info!(
                        "Atribuindo tarefa {} para o trabalhador {}",
                        task.id, worker_id
                    );
                    Response::AssignTask { task }
                } else {
                    debug!("Nenhuma tarefa disponível para o trabalhador {worker_id}");
                    Response::NoTaskAvailable
                }
            }
            Request::ReportResult { worker_id, result } => {
                info!(
                    "Recebido resultado para a tarefa {} do trabalhador {}",
                    result.task_id, worker_id
                );
                let mut tm = task_manager.lock().await;
                tm.mark_task_completed(result.task_id)?;

                let mut ra = result_aggregator.lock().await;
                ra.add_result(result)?;
                Response::Ack
            }
            Request::Heartbeat { worker_id } => {
                debug!("Recebido heartbeat do trabalhador {worker_id}");
                Response::Ack
            }
        };

        // serializa a Response para JSON e envia, adicionando um newline como delimitador
        let encoded: Vec<u8> = serde_json::to_vec(&response)?;
        reader.write_all(&encoded).await?;
        reader.write_all(b"\n").await?; // Adiciona delimitador de newline
        reader.flush().await?;
        debug!("Resposta enviada para o trabalhador: {response:?}");
    }
}
