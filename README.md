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

'secret' is used to sign JSON Web Tokens assigned to users when they login
for access to the API. It is recommened to create a random string of 512b to
be used for the secret. A random secret can be generated using the following
command:

```
openssl rand -base64 512
```

```toml
[server]
address = "127.0.0.1"
port = 8000
secret = ""

[database]
user = "database user"
host = "127.0.0.1"
port = 26572
cert_file = "/path/to/user/cert"
cert_key_file = "/path/to/user/cert/key/file"
ca_file = "/path/to/ca/file"
```
