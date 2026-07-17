use std::time::Duration;

use reqwest::redirect::Policy;

use crate::application::workspace_service::Readiness;
use crate::error::{AppError, AppResult};

pub async fn wait_for_tcp(host: &str, port: u16, timeout: Duration) -> AppResult<()> {
    if !matches!(host, "127.0.0.1" | "localhost" | "::1") {
        return Err(AppError::Other("readiness target must be local".into()));
    }
    tokio::time::timeout(timeout, tokio::net::TcpStream::connect((host, port)))
        .await
        .map_err(|_| AppError::Other("readiness timed out".into()))??;
    Ok(())
}

pub async fn wait_for_readiness(readiness: &Readiness) -> AppResult<()> {
    let (timeout, interval) = match readiness {
        Readiness::Tcp {
            timeout_seconds,
            interval_milliseconds,
            ..
        }
        | Readiness::Http {
            timeout_seconds,
            interval_milliseconds,
            ..
        } => (
            Duration::from_secs(*timeout_seconds),
            Duration::from_millis(*interval_milliseconds),
        ),
    };
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if probe(readiness, interval).await.is_ok() {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(AppError::Other("readiness timed out".into()));
        }
        tokio::time::sleep(
            interval.min(deadline.saturating_duration_since(tokio::time::Instant::now())),
        )
        .await;
    }
}

async fn probe(readiness: &Readiness, attempt_timeout: Duration) -> AppResult<()> {
    match readiness {
        Readiness::Tcp { host, port, .. } => wait_for_tcp(host, *port, attempt_timeout).await,
        Readiness::Http {
            url,
            expected_status,
            ..
        } => {
            if !is_local_http_url(url) {
                return Err(AppError::Other("readiness target must be local".into()));
            }
            let client = reqwest::Client::builder()
                .redirect(Policy::none())
                .timeout(attempt_timeout)
                .build()
                .map_err(|error| AppError::Other(format!("readiness client failed: {error}")))?;
            let response = client
                .get(url)
                .send()
                .await
                .map_err(|_| AppError::Other("readiness request failed".into()))?;
            if (expected_status[0]..=expected_status[1]).contains(&response.status().as_u16()) {
                Ok(())
            } else {
                Err(AppError::Other(
                    "readiness returned an unexpected status".into(),
                ))
            }
        }
    }
}

fn is_local_http_url(value: &str) -> bool {
    value.starts_with("http://127.0.0.1:")
        || value.starts_with("http://localhost:")
        || value.starts_with("http://[::1]:")
}

#[cfg(test)]
mod tests {
    use super::{wait_for_readiness, wait_for_tcp};
    use crate::application::workspace_service::Readiness;
    use std::time::Duration;
    #[tokio::test]
    async fn reaches_loopback_listener() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        wait_for_tcp("127.0.0.1", port, Duration::from_secs(1))
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn rejects_remote_target() {
        assert!(wait_for_tcp("example.com", 443, Duration::from_secs(1))
            .await
            .is_err());
    }
    #[tokio::test]
    async fn accepts_expected_local_http_status() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut request = [0u8; 1024];
            let _ = stream.read(&mut request).await.unwrap();
            stream
                .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n")
                .await
                .unwrap();
        });
        let readiness = Readiness::Http {
            url: format!("http://127.0.0.1:{port}/health"),
            expected_status: [200, 299],
            interval_milliseconds: 10,
            timeout_seconds: 1,
        };
        wait_for_readiness(&readiness).await.unwrap();
    }
    #[tokio::test]
    async fn retries_until_timeout() {
        let readiness = Readiness::Tcp {
            host: "127.0.0.1".into(),
            port: 1,
            interval_milliseconds: 10,
            timeout_seconds: 1,
        };
        assert!(wait_for_readiness(&readiness).await.is_err());
    }
}
