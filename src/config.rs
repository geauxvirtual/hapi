use std::io;
use std::fs::File;
use std::io::prelude::*;

pub fn read_file(file: &str) -> Result<String, io::Error> {
    let mut contents = String::new();

    File::open(file)?.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn default() -> Config {
    Config {
        server: default_server_config(),
        database: default_database_config(),
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_server_config")]
    pub server: ServerConfig,

    #[serde(default = "default_database_config")]
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_address")]
    pub address: String,

    #[serde(default = "default_server_port")]
    pub port: u64,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_name")]
    pub name: String,

    #[serde(default = "default_db_user")]
    pub user: String,

    #[serde(default = "default_db_host")]
    pub host: String,

    #[serde(default = "default_db_port")]
    pub port: u64,

    #[serde(default = "default_db_cert_file")]
    pub cert_file: String,

    #[serde(default = "default_db_cert_key_file")]
    pub cert_key_file: String,

    #[serde(default = "default_db_ca_file")]
    pub ca_file: String,
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

fn default_server_port() -> u64 {
    8000
}

fn default_database_config() -> DatabaseConfig {
    DatabaseConfig {
        name: default_db_name(),
        user: default_db_user(),
        host: default_db_host(),
        port: default_db_port(),
        cert_file: default_db_cert_file(),
        cert_key_file: default_db_cert_key_file(),
        ca_file: default_db_ca_file(),
    }
}

fn default_db_name() -> String {
    "default".to_string()
}

fn default_db_user() -> String {
    "root".to_string()
}

fn default_db_host() -> String {
    "127.0.0.1".to_string()
}

fn default_db_port() -> u64 {
    26572
}

fn default_db_cert_file() -> String {
    "".to_string()
}

fn default_db_cert_key_file() -> String {
    "".to_string()
}

fn default_db_ca_file() -> String {
    "".to_string()
}
