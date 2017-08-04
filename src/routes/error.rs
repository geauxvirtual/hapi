use rocket_contrib::{Json, Value};

use super::Response;

#[error(400)]
fn bad_request() -> Json<Value> {
    Json(json!(Response::new("error", "The request could not be understood by the server")))
}

#[error(411)]
fn length_required() -> Json<Value> {
    Json(json!(Response::new("error", "Content-Length is required")))
}

#[error(413)]
fn payload_too_large() -> Json<Value> {
    Json(json!(Response::new("error", "Maximum payload size is 10MB")))
}
