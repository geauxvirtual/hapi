use rocket::response::status;
use rocket::http::Status;

use rocket_contrib::{Json, Value};

use chrono::Utc;
use rand;
use rand::Rng;
use argon2rs::defaults::{KIB, LANES, PASSES};
use argon2rs::verifier::Encoded;
use argon2rs::{Argon2, Variant};

use hdb::platform::models::users;
use hdb::platform::models::users::NewUser;

use db::Conn;

#[derive(Deserialize)]
struct UserRequest {
    username: String,
    password: String,
}

#[post("/register", format="application/json", data="<message>")]
fn register(message: Json<UserRequest>, db: Conn) -> status::Custom<Json<Value>> {
    // Check if user already exists. Return error
    let exists = users::exists(&message.0.username, &db);
    // Create new user, return user or error
    if exists {
        return status::Custom(
            Status::Conflict,
            Json(json!({
                "status": "error",
                "reason": "Email address already exists"}))
            )
    }
    // Generate Salt
    let salt = rand::thread_rng().gen_ascii_chars().take(32).collect::<String>();
    // Generate password hash
    let a2 = Argon2::new(PASSES, LANES, KIB, Variant::Argon2d).unwrap();
    let password_hash = Encoded::new(a2, message.0.password.as_bytes(), salt.as_bytes(), b"", b"").to_u8(); 
    let new_user = NewUser {
        username: message.0.username, //Sanity check username??
        salt: salt.as_bytes().to_vec(),
        password: password_hash,
        active: true,
        created_on: Utc::now(),
    };
    let success = users::create(new_user, &db);
    if success {
        status::Custom(
            Status::Created,
            Json(json!({ "status": "ok" }))
        )
    } else {
        status::Custom(
            Status::InternalServerError,
            Json(json!({
                "status": "error",
                "reason": "error creating user"}))
        )
    }
}

#[post("/login", format="application/json", data="<message>")]
fn login(message: Json<UserRequest>, db: Conn) -> status::Custom<Json<Value>> {
    // Check if user exists
    let exists = users::exists(&message.0.username, &db);
    if !exists {
        return status::Custom(
            Status::Unauthorized,
            Json(json!({
                "status": "error",
                "reason": "username or password incorrect"
            }))
        )
    }
    // Retrieve user from the database and try to validate
    // authentication
    let user = users::get_by_username(&message.0.username, &db).unwrap();
    if !user.active {
        return status::Custom(
            Status::Unauthorized,
            Json(json!({
                "status": "error",
                "reason": "username or password incorrect"
            }))
        )
    }
    
    let a2 = Argon2::new(PASSES, LANES, KIB, Variant::Argon2d).unwrap();
    let tph = Encoded::new(a2, message.0.password.as_bytes(), &user.salt, b"", b"").to_u8();
    if tph == user.password {
       // Return authorization key upon successful authentication 
        status::Custom(
            Status::Ok,
            Json(json!({
                "status": "ok"
            }))
        )
    } else {
        status::Custom(
            Status::Unauthorized,
            Json(json!({
                "status": "error",
                "reason": "username or password incorrect"
            }))
        )
    }
}
