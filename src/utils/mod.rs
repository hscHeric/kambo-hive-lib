use std::{
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

use log::{error, info, warn};

pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Logger inicializado");
}

const DISCOVERY_PORT: u16 = 2901;
const DISCOVERY_MESSAGE: &[u8] = b"KAMBO_HIVE_DISCOVERY_REQUEST";
const RESPONSE_PREFIX: &[u8] = b"KAMBO_HIVE_HOST_IS_AT:";

pub fn discover_host() -> Result<String, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;

    info!("Enviando broadcast para encontrar host na porta {DISCOVERY_PORT}...");
    socket.send_to(DISCOVERY_MESSAGE, ("255.255.255.255", DISCOVERY_PORT))?;

    let mut buf = [0; 1024];
    match socket.recv_from(&mut buf) {
        Ok((amt, src)) => {
            let response = &buf[..amt];
            if response.starts_with(RESPONSE_PREFIX) {
                let addr_bytes = &response[RESPONSE_PREFIX.len()..];

                let host_addr = std::str::from_utf8(addr_bytes)?.trim().to_string();

                info!("Host encontrado em '{host_addr}' (respondido por {src})");
                Ok(host_addr)
            } else {
                Err("Resposta inválida do host.".into())
            }
        }
        Err(e) => {
            error!("Não foi possível encontrar um host na rede: {e}");
            Err(e.into())
        }
    }
}

pub async fn listen_for_workers(tcp_bind_addr: String) {
    let tcp_port = if let Some(port) = tcp_bind_addr.split(':').next_back() {
        port
    } else {
        error!("Endereço de bind inválido para o listener: {tcp_bind_addr}");
        return;
    };

    let socket = match UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT)) {
        Ok(s) => s,
        Err(e) => {
            error!("Falha ao escutar na porta de descoberta {DISCOVERY_PORT}: {e}");
            return;
        }
    };
    info!("Host escutando por broadcasts de descoberta na porta {DISCOVERY_PORT}");

    let mut buf = [0; 1024];
    loop {
        if let Ok((amt, worker_addr)) = socket.recv_from(&mut buf) {
            if &buf[..amt] == DISCOVERY_MESSAGE {
                info!("Requisição de descoberta recebida de {worker_addr}");

                // Descobre qual IP local usar para responder ao worker
                if let Some(local_ip) = get_local_ip_for_target(worker_addr) {
                    let response_addr = format!("{local_ip}:{tcp_port}");
                    info!("Respondendo para {worker_addr} com o endereço: {response_addr}");

                    let payload = [RESPONSE_PREFIX, response_addr.as_bytes()].concat();

                    if let Err(e) = socket.send_to(&payload, worker_addr) {
                        error!("Falha ao enviar resposta para {worker_addr}: {e}");
                    }
                } else {
                    warn!("Não foi possível determinar o IP local para responder a {worker_addr}");
                }
            }
        }
    }
}

fn get_local_ip_for_target(target_addr: SocketAddr) -> Option<std::net::IpAddr> {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket
                .connect(target_addr)
                .and_then(|()| socket.local_addr())
        })
        .map(|local_addr| local_addr.ip())
        .ok()
}
