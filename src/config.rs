use std::io;
use std::fs::File;
use std::io::prelude::*;

use hdb::platform::Config as DatabaseConfig;

pub fn read_file(file: &str) -> Result<String, io::Error> {
    let mut contents = String::new();

    File::open(file)?.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn default() -> Config {
    Config {
        server: default_server_config(),
        database: DatabaseConfig::default(),
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_server_config")]
    pub server: ServerConfig,

    #[serde(default = "DatabaseConfig::default")]
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_address")]
    pub address: String,

    #[serde(default = "default_server_port")]
    pub port: u16,
}

fn default_server_config() -> ServerConfig {
    ServerConfig {
        address: default_server_address(),
        port: default_server_port(),
    }
}

fn default_server_address() -> String {
    "127.0.0.1".to_string()
}

fn default_server_port() -> u16 {
    8000
}

