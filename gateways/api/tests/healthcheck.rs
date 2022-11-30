use std::net::TcpListener;

use api::server;

#[tokio::test]
async fn healthcheck_works() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to address");
    let port = listener.local_addr().unwrap().port();
    let app = server::setup_server(listener)
        .await
        .expect("failed to setup the server");
    tokio::spawn(app);
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
