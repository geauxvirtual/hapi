use chrono::{DateTime, Duration, Utc};

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, Outcome};

use rand;
use rand::Rng;
use argon2rs::defaults::{KIB, LANES, PASSES};
use argon2rs::verifier::Encoded;
use argon2rs::{Argon2, Variant};

#[derive(Serialize)]
pub struct AccessToken(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for AccessToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<AccessToken, ()> {
        let header_map = request.headers();
        let value = header_map.get_one("Authorization");
        match value {
            Some(val) => {
                let v: Vec<&str> = val.split_whitespace().collect();
                Outcome::Success(AccessToken(v[1].to_string()))
            },
            None => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

pub fn generate_hash(pass: String, salt: &Vec<u8>) -> Vec<u8> {
    let a2 = Argon2::new(PASSES,
                         LANES,
                         KIB,
                         Variant::Argon2d).unwrap();
    Encoded::new(a2,
                 pass.as_bytes(),
                 salt,
                 b"",
                 b"").to_u8()
}

pub fn generate_salt() -> Vec<u8> {
    random(32).as_bytes().to_vec()
}

pub struct UserToken {
    pub token: String,
    pub expires: DateTime<Utc>,
}

pub fn generate_user_token() -> UserToken {
    UserToken {
        token: random(128),
        expires: Utc::now() + Duration::seconds(3600),
    }
}

fn random(take: usize) -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .take(take)
        .collect::<String>()
}
