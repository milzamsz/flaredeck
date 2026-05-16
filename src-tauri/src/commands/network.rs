use std::time::Duration;

use tokio::net::TcpStream;

use crate::error::AppResult;

#[tauri::command]
pub async fn network_check_port(host: String, port: u16) -> AppResult<bool> {
    let addr = format!("{host}:{port}");
    match tokio::time::timeout(Duration::from_millis(1000), TcpStream::connect(addr)).await {
        Ok(Ok(_)) => Ok(true),
        Ok(Err(_)) => Ok(false),
        Err(_) => Ok(false),
    }
}
