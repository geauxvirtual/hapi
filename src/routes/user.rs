use rocket::response::status;
use rocket::http::Status;

use rocket_contrib::{Json, Value};

use chrono::Utc;

use hdb::platform::models::users;
use hdb::platform::models::users::NewUser;
use hdb::platform::models::tokens;
use hdb::platform::models::tokens::NewUserToken;

use db::Conn;
use super::Response;
use auth;

#[derive(Deserialize)]
struct UserRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthenticatedUser {
    username: String,
    access_token: String,
}

#[post("/register", format="application/json", data="<message>")]
fn register(message: Json<UserRequest>, db: Conn) -> status::Custom<Json<Value>> {
    // Check if user already exists. Return error
    let exists = users::exists(&message.0.username, &db);
    // Create new user, return user or error
    if exists {
        return status::Custom(
            Status::Conflict,
            Json(json!(Response::new("error", "Username already exists")))
        )
    }
    // Generate Salt
    let salt = auth::generate_salt();
    // Generate password hash
    let hash = auth::generate_hash(message.0.password, &salt);
    let new_user = NewUser {
        username: message.0.username, //Sanity check username??
        salt: salt,
        password: hash,
        active: true,
        created_on: Utc::now(),
    };
    let success = users::create(new_user, &db);
    if success {
        status::Custom(
            Status::Created,
            Json(json!(Response::new("ok", "User created")))
        )
    } else {
        status::Custom(
            Status::InternalServerError,
            Json(json!(Response::new("error", "error creating user")))
        )
    }
}

#[post("/login", format="application/json", data="<message>")]
fn login(message: Json<UserRequest>, db: Conn) -> status::Custom<Json<Value>> {
    // Attempt to find user in the database. Return unauthorized if no user
    // is found.
    let user = match users::get_by_username(&message.0.username, &db) {
        Ok(u) => u,
        Err(_) => return unauthorized(),
    };
    
    if !user.active {
        return unauthorized()
    }
    
    let tph = auth::generate_hash(message.0.password, &user.salt);
    if tph == user.password {
        // TODO: Check if user already has an access token and return it
        // if not expired

        // Return authorization key upon successful authentication 
        let user_token = auth::generate_user_token();
        let new_user_token = NewUserToken {
            username: user.username.clone(),
            token: user_token.token.as_bytes().to_vec(),
            expires: user_token.expires};
        let success = tokens::create(new_user_token, &db);
        if success {
            status::Custom(
                Status::Ok,
                Json(json!(AuthenticatedUser{
                    username: user.username,
                    access_token: user_token.token,
                }))
            )
        } else {
            status::Custom(
                Status::InternalServerError,
                Json(json!(Response::new("error", "Internal server error")))
            )
        }
    } else {
        unauthorized()
    }
}

fn unauthorized() -> status::Custom<Json<Value>> {
    status::Custom(
        Status::Unauthorized,
        Json(json!(Response::new("error", "username or password is incorrect")))
    )
}
