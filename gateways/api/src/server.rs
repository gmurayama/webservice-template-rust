use actix_web::{dev::Server, get, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

#[tracing::instrument]
#[get("/healthcheck")]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn setup_server(listener: TcpListener) -> eyre::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(healthcheck)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
