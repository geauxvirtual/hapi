pub mod user;

#[derive(Serialize)]
struct Response {
    status: String,
    reason: String
}

impl Response {
    fn new<S>(status: S, reason: S) -> Response
    where
        S: Into<String>
    {
        Response {
            status: status.into(),
            reason: reason.into()
        }
    }
}

#[get("/")]
pub fn index() -> &'static str {
    "Welcome to hapi"
}
