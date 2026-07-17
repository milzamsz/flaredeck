use std::process::{Command, Stdio};
use std::time::Duration;

use flaredeck_lib::application::webhook_service;

#[tokio::test]
async fn proxy_forwards_to_loopback_but_persists_only_redacted_bounded_data() {
    let root = std::env::temp_dir().join(format!("flaredeck-webhook-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&root).await.unwrap();
    let events = root.join("events.json");
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_port = origin.local_addr().unwrap().port();
    let proxy_port = available_port();
    let origin_task = tokio::spawn(async move {
        let (mut stream, _) = origin.accept().await.unwrap();
        let mut request = vec![0u8; 4096];
        let read = stream
            .readable()
            .await
            .and_then(|_| stream.try_read(&mut request))
            .unwrap();
        let request = String::from_utf8_lossy(&request[..read]);
        assert!(request.contains("CANARY_SECRET"));
        use tokio::io::AsyncWriteExt;
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
    });
    let mut proxy = Command::new(env!("CARGO_BIN_EXE_flaredeck-webhook-proxy"))
        .args([
            format!("127.0.0.1:{proxy_port}"),
            "route-test".into(),
            format!("http://127.0.0.1:{origin_port}"),
            events.to_string_lossy().into_owned(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_for_port(proxy_port).await;

    let response = reqwest::Client::new()
        .post(format!(
            "http://127.0.0.1:{proxy_port}/hook?token=CANARY_SECRET"
        ))
        .header("authorization", "Bearer CANARY_SECRET")
        .json(&serde_json::json!({ "password": "CANARY_SECRET", "safe": "ok" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 204);
    origin_task.await.unwrap();
    proxy.kill().unwrap();
    proxy.wait().unwrap();

    let captured = webhook_service::list_events(&events, "route-test", 10)
        .await
        .unwrap();
    assert_eq!(captured.len(), 1);
    let serialized = serde_json::to_string(&captured).unwrap();
    assert!(!serialized.contains("CANARY_SECRET"));
    assert!(serialized.contains("[REDACTED]"));
    assert!(webhook_service::replay_event(
        &events,
        "route-test",
        &captured[0].id,
        "https://example.com"
    )
    .await
    .is_err());
    let _ = tokio::fs::remove_dir_all(root).await;
}

#[tokio::test]
async fn proxy_rejects_oversized_body_before_forwarding_or_persisting() {
    let root = std::env::temp_dir().join(format!("flaredeck-webhook-{}", uuid::Uuid::new_v4()));
    tokio::fs::create_dir_all(&root).await.unwrap();
    let events = root.join("events.json");
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin_port = origin.local_addr().unwrap().port();
    let proxy_port = available_port();
    let mut proxy = Command::new(env!("CARGO_BIN_EXE_flaredeck-webhook-proxy"))
        .args([
            format!("127.0.0.1:{proxy_port}"),
            "route-test".into(),
            format!("http://127.0.0.1:{origin_port}"),
            events.to_string_lossy().into_owned(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_for_port(proxy_port).await;

    let response = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{proxy_port}/hook"))
        .body(vec![b'x'; 64 * 1024 + 1])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 413);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), origin.accept())
            .await
            .is_err()
    );
    assert!(webhook_service::list_events(&events, "route-test", 10)
        .await
        .unwrap()
        .is_empty());

    proxy.kill().unwrap();
    proxy.wait().unwrap();
    let _ = tokio::fs::remove_dir_all(root).await;
}

fn available_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

async fn wait_for_port(port: u16) {
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("proxy did not start");
}
