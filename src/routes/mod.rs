//pub mod user;

#[get("/")]
pub fn index() -> &'static str {
    "Welcome to hapi"
}
