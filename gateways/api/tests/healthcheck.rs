use api::{metrics::ApiMetrics, server};
use prometheus_client::registry::Registry;

#[tokio::test]
async fn healthcheck_works() {
    let mut registry = Registry::default();
    let api_metrics = ApiMetrics::new(&mut registry);

    let app = server::Server::setup(server::Settings {
        host: "127.0.0.1".to_string(),
        port: 0,
        metrics: api_metrics,
        registry,
    })
    .expect("failed to setup the server");
    let port = app.port();

    tokio::spawn(app.run());
    let client = reqwest::Client::new();

    // Act
    let response = client
        // Use the returned application address
        .get(&format!("http://localhost:{}/healthcheck", &port))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
