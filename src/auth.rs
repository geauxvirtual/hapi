use rand;
use rand::Rng;
use argon2rs::defaults::{KIB, LANES, PASSES};
use argon2rs::verifier::Encoded;
use argon2rs::{Argon2, Variant};

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
    rand::thread_rng()
        .gen_ascii_chars()
        .take(32)
        .collect::<String>()
        .as_bytes()
        .to_vec()
}
