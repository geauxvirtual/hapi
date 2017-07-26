use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};

use hdb::platform::{Config, Database, Pool, PoolConnection, PlatformConnection};

pub fn init_pool(config: Config) -> Pool {
    Database::new(config).pool()
}

pub struct Conn(pub PoolConnection);

impl<'a, 'r> FromRequest<'a, 'r> for Conn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Conn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Conn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

impl Deref for Conn {
    type Target = PlatformConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
