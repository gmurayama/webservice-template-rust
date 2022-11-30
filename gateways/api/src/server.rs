use actix_web::{dev::Server, get, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

#[tracing::instrument]
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

pub async fn setup_server(listener: TcpListener) -> eyre::Result<Server> {
    let server = HttpServer::new(|| App::new().service(hello))
        .listen(listener)?
        .run();

    Ok(server)
}
