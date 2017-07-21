// hapi is the API server.

extern crate toml;
extern crate serde;
#[macro_use] extern crate serde_derive;

extern crate clap;
extern crate hdb;

mod config;
mod cli;

use config::Config;

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
    // TODO: Configure database connection
    // TODO: Start Rocket
}
