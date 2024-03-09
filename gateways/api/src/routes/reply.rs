use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use application::messages::{self, ReplyError};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct ReplyRequest {
    message: String,
}

#[tracing::instrument(name = "gateways.api.routes.reply")]
pub async fn reply(request: web::Json<ReplyRequest>) -> impl Responder {
    match messages::reply(&request.message) {
        Ok(message) => HttpResponse::Ok().json(json!({
            "message": message
        })),
        Err(err) => match err {
            ReplyError::UnknownMessage(_) => HttpResponse::UnprocessableEntity().json(json!({
                "message": format!("{err}")
            })),
        },
    }
}
