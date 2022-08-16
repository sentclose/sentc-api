# Sentc server

From sentclose

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

## Add new endpoint

1. add to the route file
2. add to the app endpoint enum in: `server_crates/server_api/src/customer_app/app_util.rs`
3. add the endpoint to the app options:
   1. insert it in the db table: `app_options`
   2. add new endpoint in select app options and create app / update app options in `server_crates/server_api/src/customer_app/app_model.rs`
   3. add to the AppOptions input incl. the default fn in: `server_crates/server_api_common/src/app.rs`

## Entity and common data

When creating a new return entity which should returned to the client via json, then create an entity struct in the entity mod.
After this, create a data struct in the common data mod. Make sure both are sync. 

A Tip: use into trait for the entity but don't use the actual into. When the data struct or the entity struct changed, rust compiler will err.