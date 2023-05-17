# Sentc server

From sentclose

This is the server api which manages the user, groups and keys. It comes with mysql/mariadb or sqlite support. 

## Commands

### Default with maria db

Running the server
````shell
cargo run
````

Running release
````shell
cargo run --release
````

Integration tests. A maria db connection is needed.
````shell
cargo test  --test '*' -- --test-threads=1
````

Unit tests
````shell
cargo test --lib -- --test-threads=1
````

Test all
````shell
cargo test  -- --test-threads=1
````

### Sqlite

Running the server
````shell
cargo run --features=sqlite --no-default-features
````

Running release
````shell
cargo run --features=sqlite --no-default-features --release
````

Integration tests. A path to the sqlite db is needed in as env
````shell
cargo test --features=sqlite --no-default-features  --test '*' -- --test-threads=1
````

Unit tests
````shell
cargo test --lib --features=sqlite --no-default-features -- --test-threads=1
````

Test all
````shell
cargo test --features=sqlite --no-default-features -- --test-threads=1
````

## Dev

For dev, it is recommended to compile sqlite into a new target dir.

````shell
cargo run --features=sqlite --no-default-features --target-dir=target_sqlite
````