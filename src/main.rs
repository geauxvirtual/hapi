#![feature(plugin)]
#![plugin(rocket_codegen)]
// hapi is the API server.

// external libs
extern crate argon2rs;
extern crate chrono;
extern crate clap;
extern crate jsonwebtoken as jwt;
extern crate multipart;
extern crate rand;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate toml;
extern crate uuid;

// Platform libs
extern crate hdb;

mod auth;
mod cli;
mod config;
mod db;
mod file;
mod routes;

use std::fs;
use std::path::Path;

use config::Config;
use rocket::config::Config as RocketConfig;
use rocket::config::Environment;

fn main() {
    //let mut config = config::default();
    // Read commandline arguments
    let matches = cli::new().get_matches();
    // Use config passed in on cli or default to default location:
    // /etc/hydra/hapi/config.toml
    let config_file = matches
        .value_of("config")
        .unwrap_or("/etc/hydra/hapi/config.toml");
    // TODO: Proper logging
    println!("Using config file: {}", config_file);
    let config: Config = match config::read_file(config_file) {
        Ok(s) => toml::from_str(&s).unwrap(),
        Err(e) => {
            eprintln!("Error {} with config file {}", e, config_file);
            println!("Error using config file {}. Using default config instead.",
                     config_file);
            config::default()
        }
    };
    println!("{:?}", config);

    // Create directory to store user files
    // TODO: Configure permissions on directory
    fs::create_dir_all(Path::new(&config.server.file_dir)).unwrap();

    // Create database connection pool
    let pool = db::init_pool(config.database);

    // Configure and start Rocket
    let server_config = RocketConfig::build(Environment::Development)
        .address(config.server.address.clone())
        .port(config.server.port)
        .unwrap();
    rocket::custom(server_config, true)
        .manage(pool)
        .manage(config.server)
        .mount("/", routes![routes::index])
        .mount("/users", routes![routes::user::register,
                                routes::user::login,
                                routes::user::delete,
                                routes::user::import])
        .catch(errors![routes::error::bad_request,
                       routes::error::length_required,
                       routes::error::payload_too_large])
        .launch();
}
