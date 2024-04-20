use actix_web::{error, web, HttpResponse};
use application::messages::{self, ReplyError};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct ReplyRequest {
    message: String,
}

#[tracing::instrument(name = "gateways.api.routes.reply")]
pub async fn reply(request: web::Json<ReplyRequest>) -> actix_web::Result<HttpResponse> {
    match messages::reply(&request.message) {
        Ok(message) => Ok(HttpResponse::Ok().json(json!({
            "message": message
        }))),
        Err(err) => Err(match err {
            ReplyError::UnknownMessage(_) => error::ErrorInternalServerError(json!({
                "message": format!("{err}")
            })),
        }),
    }
}
