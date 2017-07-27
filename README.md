hapi - API Server
=================

Build
-----

```
rustup run nightly cargo build
```

Run
---

```
rustup run nightly cargo run -- -c <configuration path>
```

Configuration
-------------

Server and database sections are optional. If not included in the configuration
file, default values will be used. The server section contains the address
and port to use for starting the API server. The database section contains
the configuration information used for connecting to a CockroachDB instance. 

```toml
[server]
address = "127.0.0.1"
port = 8000

[database]
user = "database user"
host = "127.0.0.1"
port = 26572
cert_file = "/path/to/user/cert"
cert_key_file = "/path/to/user/cert/key/file"
ca_file = "/path/to/ca/file"
```
