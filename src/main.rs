#![feature(plugin)]
#![plugin(rocket_codegen)]
// hapi is the API server.

extern crate toml;
extern crate serde;
#[macro_use] extern crate serde_derive;

extern crate clap;
extern crate hdb;

extern crate rocket;

mod config;
mod cli;
mod routes;

use config::Config;
use hdb::platform::Database;
use rocket::config::Config as RocketConfig;
use rocket::config::Environment;

fn main() {
    //let mut config = config::default();
    // Read commandline arguments
    let matches = cli::new().get_matches();
    // Use config passed in on cli or default to default location:
    // /etc/hydra/hapi/config.toml
    let config_file = matches.value_of("config").unwrap_or("/etc/hydra/hapi/config.toml");
    // TODO: Proper logging
    println!("Using config file: {}", config_file);
    let config: Config = match config::read_file(config_file) {
        Ok(s) => toml::from_str(&s).unwrap(),
        Err(e) => {
            eprintln!("Error {} with config file {}", e, config_file);
            println!("Error using config file {}. Using default config instead.", config_file);
            config::default()
        }
    };
    println!("{:?}", config);
    
    // Create database connection pool
    let db = Database::new(config.database);
    let pool = db.pool();

    // Configure and start Rocket
    let server_config = RocketConfig::build(Environment::Development)
        .address(config.server.address)
        .port(config.server.port)
        .unwrap();
    rocket::custom(server_config, true)
        .manage(pool)
        .mount("/", routes![routes::index])
        .launch();
}
