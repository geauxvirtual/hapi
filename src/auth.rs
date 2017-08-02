use chrono::{Duration, Utc};

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, Outcome};

use rand;
use rand::Rng;
use argon2rs::defaults::{KIB, LANES, PASSES};
use argon2rs::verifier::Encoded;
use argon2rs::{Argon2, Variant};

use jwt;
use jwt::{encode, decode, Header, Validation};

pub type Secret = String;

#[derive(Serialize)]
pub struct AccessToken(pub String);

impl<'a, 'r> FromRequest<'a, 'r> for AccessToken {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<AccessToken, ()> {
        let header_map = request.headers();
        let value = header_map.get_one("Authorization");
        match value {
            Some(val) => {
                // Bearer should proceed the access token
                // If the token does not start at index 7, an error
                // will be returned when trying to validate the token
                let (_, v) = val.split_at(7);
                Outcome::Success(AccessToken(v.to_string()))
            },
            None => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Claim {
    // User id token is issued for
    sub: String,
    // Time token was issued
    iat: i64,
    // Time token expires.
    exp: i64,
}

impl Claim {
    fn new(sub: &str, iat: i64, exp: i64) -> Claim {
        Claim {
            sub: sub.to_string(),
            iat: iat,
            exp: exp,
        }
    }
}

pub struct UserToken;

impl UserToken {
    pub fn new(sub: &str, secret: &Secret) -> Result<String, jwt::errors::Error> {
        let now = Utc::now();
        let expires = now + Duration::seconds(3600);
        let claim = Claim::new(sub, now.timestamp(), expires.timestamp());
        encode(&Header::default(), &claim, secret.as_bytes())
    }

    pub fn validate(token: &str, secret: &Secret, sub: &str) -> bool {
        let validation = Validation {
            sub: Some(sub.to_string()),
            ..Default::default()
        };
        match decode::<Claim>(&token, secret.as_bytes(), &validation) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}


// Functions for hashing user passwords
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

fn random(take: usize) -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .take(take)
        .collect::<String>()
}
