use rocket::request::State;
use rocket::response::status;
use rocket::http::Status;

use rocket_contrib::{Json, Value, UUID};

use chrono::Utc;
use uuid::Uuid;

use hdb::platform::models::users;
use hdb::platform::models::users::NewUser;
use hdb::platform::models::tokens;
use hdb::platform::models::tokens::NewUserToken;

use db::Conn;
use super::Response;
use auth;
use auth::{AccessToken, UserToken, Secret};

#[derive(Deserialize)]
struct UserRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct AuthenticatedUser {
    user_id: Uuid,
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
fn login(message: Json<UserRequest>, db: Conn, secret: State<Secret>) -> status::Custom<Json<Value>> {
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
        let user_token = match tokens::get_by_user_id(&user.id, &db) {
            // If user already has access_token retrieve it from the db
            // and return it
            Ok(ut) => {
                let token = match String::from_utf8(ut.token) {
                    Ok(t) => t,
                    Err(_) => return internal_server_error(),
                };
                // Check to see if token is valid
                if UserToken::validate(&token, &secret, &user.id.to_string()) {
                    token
                // If the current user token is invalid, generate a new
                // user token and return it
                } else {
                    let user_token = match UserToken::new(&user.id.to_string(), &secret) {
                        Ok(ut) => ut,
                        Err(_) => return internal_server_error(),
                    };
                    let success = tokens::update(&ut.id,
                                                 &user_token.as_bytes().to_vec(),
                                                 &db);
                    if success {
                        user_token
                    } else {
                        return internal_server_error();
                    }
                }
            },

            // If user does not have an access token, then create a new
            // access_token for the user
            Err(_) => {
                let user_token = match UserToken::new(&user.id.to_string(), &secret) {
                    Ok(ut) => ut,
                    Err(_) => return internal_server_error(),
                };
                let new_user_token = NewUserToken {
                    user_id: user.id,
                    token: user_token.as_bytes().to_vec(),
                };
                let success = tokens::create(new_user_token, &db);
                if success {
                    user_token
                } else {
                    return internal_server_error();
                }
            }
        };

        // Return user_id, username, and access_token with successful login
        status::Custom(
            Status::Ok,
            Json(json!(AuthenticatedUser{
                user_id: user.id,
                username: user.username,
                access_token: user_token,
            }))
        )
    } else {
        unauthorized()
    }
}

#[delete("/<id>")]
fn delete(access_token: AccessToken, id: UUID, db: Conn, secret: State<Secret>) -> status::Custom<Json<Value>> {
    // Validate received token
    if UserToken::validate(&access_token.0, &secret, &id.to_string()) {
        if users::inactivate(&id, &db) {
            let token = tokens::get_by_user_id(&id, &db).unwrap();
            if tokens::delete(&token.id, &db) {
                status::Custom(
                    Status::Accepted,
                    Json(json!(Response::new("accepted", "user inactive")))
                )
            } else {
                internal_server_error()
            }
        } else {
            internal_server_error()
        }
    } else {
    // Invalid token passed to us.
        unauthorized_token()
    }
}

fn unauthorized() -> status::Custom<Json<Value>> {
    status::Custom(
        Status::Unauthorized,
        Json(json!(Response::new("error", "username or password is incorrect")))
    )
}

fn unauthorized_token() -> status::Custom<Json<Value>> {
    status::Custom(
        Status::Unauthorized,
        Json(json!(Response::new("error", "unauthorized")))
    )
}

fn internal_server_error() -> status::Custom<Json<Value>> {
    status::Custom(
        Status::InternalServerError,
        Json(json!(Response::new("error", "internal server error")))
    )
}
