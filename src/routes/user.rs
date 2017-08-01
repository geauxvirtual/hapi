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
use auth::AccessToken;

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
        let user_token = match tokens::get_by_user_id(&user.id, &db) {
            // If user already has access_token retrieve it from the db
            // and return it
            Ok(ut) => {
                // Check to see if token is valid
                if Utc::now() < ut.expires {
                    auth::UserToken {
                        token: String::from_utf8(ut.token).unwrap(),
                        expires: ut.expires,
                    }
                // Generate new token and update database entry
                } else {
                    let user_token  = auth::generate_user_token();
                    let success = tokens::update(&ut.id,
                                                 &user_token.token.as_bytes().to_vec(),
                                                 &user_token.expires,
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
                let user_token = auth::generate_user_token();
                let new_user_token = NewUserToken {
                    user_id: user.id,
                    token: user_token.token.as_bytes().to_vec(),
                    expires: user_token.expires,
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
                access_token: user_token.token,
            }))
        )
    } else {
        unauthorized()
    }
}

#[delete("/<id>")]
fn delete(access_token: AccessToken, id: UUID, db: Conn) -> status::Custom<Json<Value>> {
    // Try to retrieve access token for provied id. If no token found,
    // return unauthorized
    let token = match tokens::get_by_user_id(&id, &db) {
        Ok(t) => t,
        Err(_) => return unauthorized_token(),
    };
    // Check to see if access token retrieved matches access_token passed
    // with request. Return unauthorized if they do not match
    if access_token.0.as_bytes().to_vec() != token.token {
        println!("1");
        return unauthorized_token();
    }
    // Check to see if token retrieved is still valid. Remove token and
    // return unauthorized if token is invalid.
    if Utc::now() > token.expires {
        println!("2");
        let success = tokens::delete(&token.id, &db);
        if success {
            return unauthorized_token();
        } else {
            return internal_server_error();
        }
    }
    
    // Token matches saved token and is valid. Mark user as inactive.
    // TODO: Marking user inactive and deleting any access tokens should
    // be in a transaction.
    let success = users::inactivate(&id, &db);
    if success {
        tokens::delete(&token.id, &db);
        println!("3");
        status::Custom(
            Status::Ok,
            Json(json!(Response::new("ok", "user inactive")))
        )
    } else {
        println!("4");
        internal_server_error()
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
