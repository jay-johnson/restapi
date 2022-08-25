//! # Rust Rest API Stack with User Management
//!
//! A secure-by-default rest api stack implemented with hyper, tokio, bb8 and postgres with prometheus for monitoring. This project is focused on providing end-to-end encryption by default for 12-factor applications looking to customize functionality using environment variables as needed. Includes a working user management and authentication backend written in postgresql with async S3 uploading for POST-ed data files.
//!
//! # Examples
//!
//! Please see the [restapi/examples/server.rs](https://github.com/jay-johnson/restapi/blob/main/examples/server.rs) for developing your own rest api.
//!
//! ## Overview
//!
//! ### User
//!
//! - User password reset and user email change support using one-time-use tokens that are stored in postgres.
//! - Users can upload and manage files stored on AWS S3 (assuming valid credentials are loaded outside this rust project).
//! - User passwords are hashed using [argon2](https://docs.rs/argon2/latest/argon2/).
//!
//! ### Auth
//!
//! - User authentication enabled by default
//! - Default JWT signing keys included with [documentation for building new keys as needed](https://github.com/jay-johnson/restapi/tree/main/jwt).
//!
//! ### Database
//!
//! - The rest api server utilizes postgres with a [bb8 client threadpool](https://github.com/djc/bb8).
//! - The postgres database requires each client connection include the postgres tls certificate authority file for encrypting data in-transit.
//! - Includes [pg4admin](https://www.pgadmin.org/docs/pgadmin4/latest/index.html) for database management in a browser (deployed with docker compose).
//!
//! ### TLS Encryption
//!
//! - Includes a tls asset generator tool ([./certs/generate-tls-assets.sh](https://github.com/jay-johnson/restapi/blob/main/certs/generate-tls-assets.sh)) for building self-signed tls assets (requires docker).
//!
//! #### Ingress
//!
//! Component        | Status
//! ---------------- | ------
//! Rest API Server  | Listening for encrypted client connections on tcp port **3000**
//! Postgres         | Listening for encrypted client connections on tcp port **5432** (tls Certificate Authority required)
//! pgAdmin          | Listening for encrypted HTTP client connections on tcp port **5433**
//!
//! ## Getting Started
//!
//! ### Clone the repo
//!
//! ```bash
//! git clone https://github.com/jay-johnson/restapi
//! cd restapi
//! ```
//!
//! ### Generate TLS Assets
//!
//! The repository [restapi](https://github.com/jay-johnson/restapi) includes default tls assets, but for security purposes you should generate your own. Please refer to the [Generate TLS Assets doc](./certs/README.md) for more information.
//!
//! Here's how to generate them under the ``./certs`` directory:
//!
//! ```bash
//! cd certs
//! ./generate-tls-assets.sh -f -c ./configs/dev-network.yml
//! cd ..
//! ```
//!
//! <a href="https://asciinema.org/a/473131?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473131.png"/></a>
//!
//! ### Generate JWT Keys
//!
//! This repo includes default JWT signing keys, but you should generate your own signing keys under the ``./jwt`` directory with these commands:
//!
//! ```bash
//! cd jwt
//! openssl ecparam -name prime256v1 -genkey -out private-key.pem
//! openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out private-key-pkcs8.pem
//! openssl ec -in private-key.pem -pubout -out public-key.pem
//! cd ..
//! ```
//!
//! <a href="https://asciinema.org/a/473132?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473132.png"/></a>
//!
//! Please refer to the [How to build JWT private and public keys for the jsonwebtokens crate doc](./certs/README.md) for more information.
//!
//! ### Build the Postgres docker image
//!
//! Please refer to the [Build and Deploy a Secured Postgres backend doc](./docker/db/README.md) for more information.
//!
//! <a href="https://asciinema.org/a/473134?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473134.png"/></a>
//!
//! ### Build API Server
//!
//! ```bash
//! cargo build --example server
//! ```
//!
//! ### Run API Server
//!
//! ```bash
//! export RUST_BACKTRACE=1 && export RUST_LOG=info && ./target/debug/examples/server
//! ```
//!
//! ## Environment Variables
//!
//! ### Rest API
//!
//! Environment Variable  | Default
//! --------------------- | -------
//! SERVER_NAME_API       | api
//! SERVER_NAME_LABEL     | rust-restapi
//! API_ENDPOINT          | 0.0.0.0:3000
//! API_TLS_DIR           | ./certs/tls/api
//! API_TLS_CA            | ./certs/tls/api/api-ca.pem
//! API_TLS_CERT          | ./certs/tls/api/api.crt
//! API_TLS_KEY           | ./certs/tls/api/api.key
//!
//! ### User Email Verification
//!
//! Environment Variable                   | Default
//! -------------------------------------- | -------
//! USER_EMAIL_VERIFICATION_REQUIRED       | "0"
//! USER_EMAIL_VERIFICATION_ENABLED        | "1"
//! USER_EMAIL_VERIFICATION_EXP_IN_SECONDS | "2592000"
//!
//! ### User One-Time-Use Token Expiration for Password Recovery
//!
//! Environment Variable    | Default
//! ----------------------- | -------
//! USER_OTP_EXP_IN_SECONDS | "2592000"
//!
//! ### Postgres Database
//!
//! Environment Variable  | Default
//! --------------------- | -------
//! DB_NAME               | mydb
//! POSTGRES_USERNAME     | datawriter
//! POSTGRES_PASSWORD     | "123321"
//! POSTGRES_ENDPOINT     | 0.0.0.0:5432
//! POSTGRES_TLS_DIR      | ./certs/tls/postgres
//! POSTGRES_TLS_CA       | ./certs/tls/postgres/postgres-ca.pem
//! POSTGRES_TLS_CERT     | ./certs/tls/postgres/postgres.crt
//! POSTGRES_TLS_KEY      | ./certs/tls/postgres/postgres.key
//! POSTGRES_DB_CONN_TYPE | postgresql
//!
//! ### S3
//!
//! Environment Variable | Default
//! -------------------- | -------
//! S3_DATA_BUCKET       | YOUR_BUCKET
//! S3_DATA_PREFIX       | /rust-restapi/tests
//! S3_STORAGE_CLASS     | STANDARD
//! S3_DATA_UPLOAD_TO_S3 | "0"
//!
//! ### JWT
//!
//! Environment Variable                 | Default
//! ------------------------------------ | -------
//! TOKEN_EXPIRATION_SECONDS_INTO_FUTURE | "2592000"
//! TOKEN_ORG                            | example.org
//! TOKEN_HEADER                         | Bearer
//! TOKEN_ALGO_PRIVATE_KEY               | ./jwt/private-key-pkcs8.pem
//! TOKEN_ALGO_PUBLIC_KEY                | ./jwt/public-key.pem
//! SERVER_PKI_DIR_JWT                   | ./jwt
//! SERVER_PASSWORD_SALT                 | 78197b60-c950-4339-a52c-053165a04764
//!
//! ### Rust
//!
//! Environment Variable | Default
//! -------------------- | -------
//! RUST_BACKTRACE       | "1"
//! RUST_LOG             | info
//!
//! ### Debug
//!
//! Environment Variable | Default
//! -------------------- | -------
//! DEBUG                | "1"
//!
//! ## Docker Builds
//!
//! ### Build Base Image
//!
//! This will build an initial base image inside a docker container. Note: this base image will **not** work on a different cpu chipset because the openssl libraries are compiled within the image for this base image.
//!
//! ```bash
//! ./build-base.sh
//! ```
//!
//! ### Build Derived Image
//!
//! By reusing the base image, this derived image only needs to recompile the server. With minimal code changes, this is a much faster build than the base image build.
//!
//! ```bash
//! ./build-derived.sh
//! ```
//!
//! ## Kubernetes
//!
//! ### Helm Chart
//!
//! #### Deploy TLS Assets into Kubernetes
//!
//! ```bash
//! ./deploy-tls-assets.sh
//! ```
//!
//! #### Deploy the Rust Rest API into Kubernetes
//!
//! By default this uses ``jayjohnson/rust-restapi`` image by default
//!
//! ```bash
//! helm install -n default rust-restapi ./charts/rust-restapi
//! ```
//!
//! ## Monitoring
//!
//! ### Prometheus
//!
//! This section assumes you have a working prometheus instance already running inside kubernetes. Below is the Prometheus ``scrape_config`` to monitor the rest api deployment replica(s) within kubernetes. Note this config also assumes the api chart is running in the ``default`` namespace:
//!
//! ```yaml
//! scrape_configs:
//! - job_name: rust-restapi
//!   scrape_interval: 10s
//!   scrape_timeout: 5s
//!   metrics_path: /metrics
//!   scheme: https
//!   tls_config:
//!     insecure_skip_verify: true
//!   static_configs:
//!   - targets:
//!     - rust-restapi.default.svc.cluster.local:3000
//! ```
//!
//! ## Supported APIs
//!
//! Here are the supported json contracts for each ``Request`` and ``Response`` based off the url. Each client request is handled by the [`handle_requests`](crate::handle_request::handle_request) and returned as a response back to the client (serialization using ``serde_json``)
//!
//! ### User APIs
//!
//! #### Create User
//!
//! Create a single ``users`` record for the new user
//!
//! - URL path: ``/user``
//! - Method: ``POST``
//! - Handler: [`create_user`](crate::requests::user::create_user::create_user)
//! - Request: [`ApiReqUserCreate`](crate::requests::user::create_user::ApiReqUserCreate)
//! - Response: [`ApiResUserCreate`](crate::requests::user::create_user::ApiResUserCreate)
//!
//! #### Update User
//!
//! Update supported ``users`` fields (including change user email and password)
//!
//! - URL path: ``/user``
//! - Method: ``PUT``
//! - Handler: [`update_user`](crate::requests::user::update_user::update_user)
//! - Request: [`ApiReqUserUpdate`](crate::requests::user::update_user::ApiReqUserUpdate)
//! - Response: [`ApiResUserUpdate`](crate::requests::user::update_user::ApiResUserUpdate)
//!
//! #### Get User
//!
//! Get a single user by ``users.id`` - by default, a user can only get their own account details
//!
//! - URL path: ``/user/USERID``
//! - Method: ``GET``
//! - Handler: [`get_user`](crate::requests::user::get_user::get_user)
//! - Request: [`ApiReqUserGet`](crate::requests::user::get_user::ApiReqUserGet)
//! - Response: [`ApiResUserGet`](crate::requests::user::get_user::ApiResUserGet)
//!
//! #### Delete User
//!
//! Delete a single ``users`` record (note: this does not delete the db record, just sets the ``users.state`` to inactive ``1``)
//!
//! - URL path: ``/user``
//! - Method: ``DELETE``
//! - Handler: [`delete_user`](crate::requests::user::delete_user::delete_user)
//! - Request: [`ApiReqUserDelete`](crate::requests::user::delete_user::ApiReqUserDelete)
//! - Response: [`ApiResUserDelete`](crate::requests::user::delete_user::ApiResUserDelete)
//!
//! #### Search Users in the db
//!
//! Search for matching ``users`` records in the db
//!
//! - URL path: ``/user/search``
//! - Method: ``POST``
//! - Handler: [`search_users`](crate::requests::user::search_users::search_users)
//! - Request: [`ApiReqUserSearch`](crate::requests::user::search_users::ApiReqUserSearch)
//! - Response: [`ApiResUserSearch`](crate::requests::user::search_users::ApiResUserSearch)
//!
//! #### Create One-Time-Use Password Reset Token (OTP)
//!
//! Create a one-time-use password reset token that allows a user to change their ``users.password`` value by presenting the token
//!
//! - URL path: ``/user/password/reset``
//! - Method: ``POST``
//! - Handler: [`create_otp`](crate::requests::user::create_otp::create_otp)
//! - Request: [`ApiReqUserCreateOtp`](crate::requests::user::create_otp::ApiReqUserCreateOtp)
//! - Response: [`ApiResUserCreateOtp`](crate::requests::user::create_otp::ApiResUserCreateOtp)
//!
//! #### Consume a One-Time-Use Password Reset Token (OTP)
//!
//! Consume a one-time-use password and change the user's ``users.password`` value to the new argon2-hashed password
//!
//! - URL path: ``/user/password/change``
//! - Method: ``POST``
//! - Handler: [`consume_user_otp`](crate::requests::user::consume_user_otp::consume_user_otp)
//! - Request: [`ApiReqUserConsumeOtp`](crate::requests::user::consume_user_otp::ApiReqUserConsumeOtp)
//! - Response: [`ApiResUserConsumeOtp`](crate::requests::user::consume_user_otp::ApiResUserConsumeOtp)
//!
//! #### Verify a User's email
//!
//! Consume a one-time-use verification token and change the user's ``users.verified`` value verified (``1``)
//!
//! - URL path: ``/user/verify``
//! - Method: ``GET``
//! - Handler: [`verify_user`](crate::requests::user::verify_user::verify_user)
//! - Request: [`ApiReqUserVerify`](crate::requests::user::verify_user::ApiReqUserVerify)
//! - Response: [`ApiResUserVerify`](crate::requests::user::verify_user::ApiResUserVerify)
//!
//! ### User S3 APIs
//!
//! #### Upload a file asynchronously to AWS S3 and store a tracking record in the db
//!
//! Upload a local file on disk to AWS S3 asynchronously and store a tracking record in the ``users_data`` table. The documentation refers to this as a ``user data`` or ``user data file`` record.
//!
//! - URL path: ``/user/data``
//! - Method: ``POST``
//! - Handler: [`upload_user_data`](crate::requests::user::upload_user_data::upload_user_data)
//! - Request: [`ApiReqUserUploadData`](crate::requests::user::upload_user_data::ApiReqUserUploadData)
//! - Response: [`ApiResUserUploadData`](crate::requests::user::upload_user_data::ApiResUserUploadData)
//!
//! #### Update an existing user data file record for a file stored in AWS S3
//!
//! Update the ``users_data`` tracking record for a file that exists in AWS S3
//!
//! - URL path: ``/user/data``
//! - Method: ``PUT``
//! - Handler: [`update_user_data`](crate::requests::user::update_user_data::update_user_data)
//! - Request: [`ApiReqUserUpdateData`](crate::requests::user::update_user_data::ApiReqUserUpdateData)
//! - Response: [`ApiResUserUpdateData`](crate::requests::user::update_user_data::ApiResUserUpdateData)
//!
//! #### Search for existing user data files from the db
//!
//! Search for matching records in the ``users_data`` db based off the request's values
//!
//! - URL path: ``/user/data/search``
//! - Method: ``POST``
//! - Handler: [`search_user_data`](crate::requests::user::search_user_data::search_user_data)
//! - Request: [`ApiReqUserSearchData`](crate::requests::user::search_user_data::ApiReqUserSearchData)
//! - Response: [`ApiResUserSearchData`](crate::requests::user::search_user_data::ApiResUserSearchData)
//!
//! ### User Authentication APIs
//!
//! #### User Login
//!
//! Log the user in and get a json web token (jwt) back for authentication on subsequent client requests
//!
//! - URL path: ``/login``
//! - Method: ``POST``
//! - Handler: [`login_user`](crate::requests::auth::login_user::login_user)
//! - Request: [`ApiReqUserLogin`](crate::requests::auth::login_user::ApiReqUserLogin)
//! - Response: [`ApiResUserLogin`](crate::requests::auth::login_user::ApiResUserLogin)
//!
//! # Interation Test Guide
//!
//! This project focused on integration tests for v1 instead of only rust tests (specifically everything has been tested with **curl**):
//!
//! Please refer to the latest [Integration Tests Using curl Guide](https://github.com/jay-johnson/restapi/blob/tests/integration-using-curl.md)
//!
//! ## Build and run the example server
//!
//! ```bash
//! cargo build --example server && export RUST_BACKTRACE=1 && export RUST_LOG=info && ./target/debug/examples/server
//! ```
//!
//! # Integration Tests Using curl Guide
//!
//! ## Set up bash curl tests
//!
//! ```bash
//! export API_TLS_DIR="./certs/tls/api"
//! export TLS_ARGS="--cacert ${API_TLS_DIR}/api-ca.pem \
//!     --cert ${API_TLS_DIR}/api.crt \
//!     --key ${API_TLS_DIR}/api.key"
//! ```
//!
//! ## User APIs
//!
//! ### Login (user does not exist yet)
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/login" \
//!     -XPOST \
//!     -d '{"email":"user@email.com","password":"12345"}' | jq
//! ```
//!
//! ### Create user
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user" \
//!     -XPOST \
//!     -d '{"email":"user@email.com","password":"12345"}' | jq
//! ```
//!
//! ### Login and save the token as an env variable
//!
//! ```bash
//! export TOKEN=$(curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/login" \
//!     -XPOST \
//!     -d '{"email":"user@email.com","password":"12345"}' | jq -r '.token')
//! ```
//!
//! ### Get user
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user/1" \
//!     -XGET \
//!     -H "Bearer: ${TOKEN}" | jq
//! ```
//!
//! ### Update user
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user" \
//!     -H "Bearer: ${TOKEN}" \
//!     -XPUT \
//!     -d '{"user_id":1,"email":"somenewemail@gmail.com","password":"321123","state":0}'
//! ```
//!
//! ### Change user password
//!
//! #### Change to a new password
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user" \
//!     -H "Bearer: ${TOKEN}" \
//!     -XPUT \
//!     -d '{"user_id":1,"password":"12345a"}' | jq
//! ```
//!
//! #### Change password back to the original
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user" \
//!     -H "Bearer: ${TOKEN}" \
//!     -XPUT \
//!     -d '{"user_id":1,"password":"12345"}' | jq
//! ```
//!
//! ### Create a one-time-use-password (otp) allowing a user to reset their users.password from the users.email
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user/password/reset" \
//!     -H "Bearer: ${TOKEN}" \
//!     -XPOST \
//!     -d '{"user_id":1,"email":"user@email.com"}' | jq
//! ```
//!
//! ### Consume user one-time-use-password token to reset the users.password (otp)
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user/password/change" \
//!     -H "Bearer: ${TOKEN}" \
//!     -XPOST \
//!     -d '{"user_id":1,"email":"user@email.com"}' | jq
//! ```
//!
//! ### Change user email
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user" \
//!     -H "Bearer: ${TOKEN}" \
//!     -XPUT \
//!     -d '{"user_id":1,"email":"unique@gmail.com"}' | jq
//! ```
//!
//! ### Verify user email
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user/verify?u=1&t=2" | jq
//! ```
//!
//! ### Search user (token must be for the POST-ed user id)
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user/search" \
//!     -XPOST \
//!     -H "Bearer: ${TOKEN}" \
//!     -d '{"email":"user","user_id":1}' | jq
//! ```
//!
//! ### Delete user
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user" \
//!     -XDELETE \
//!     -d '{"email":"user@email.com","user_id":1}' \
//!     -H "Content-type: application/json" \
//!     -H "Bearer: ${TOKEN}" | jq
//! ```
//!
//! ## JWT (json web tokens)
//!
//! ### Configurable JWT Environment Variables
//!
//! #### Header key for the token:
//!
//! ```bash
//! export TOKEN_HEADER="Bearer"
//! ```
//!
//! #### Token Org (embedded in the jwt)
//!
//! ```bash
//! export TOKEN_ORG="Org Name";
//! ```
//!
//! #### Token Lifetime Duration
//!
//! ```bash
//! # 30 days
//! export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=2592000;
//! # 7 days
//! export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=604800;
//! # 1 day
//! export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=86400;
//! ```
//!
//! #### JWT Signing Keys
//!
//! ```bash
//! export TOKEN_ALGO_KEY_DIR="./jwt"
//! export TOKEN_ALGO_PRIVATE_KEY_ORG="${TOKEN_ALGO_KEY_DIR}/private-key.pem"
//! export TOKEN_ALGO_PRIVATE_KEY="${TOKEN_ALGO_KEY_DIR}/private-key-pkcs8.pem"
//! export TOKEN_ALGO_PUBLIC_KEY="${TOKEN_ALGO_KEY_DIR}/public-key.pem"
//! ```
//!
//! ##### Generate your own jwt keys with these commands
//!
//! These commands were tested on ubuntu 21.10 using bash:
//!
//! ```bash
//! openssl ecparam -name prime256v1 -genkey -out "${TOKEN_ALGO_PRIVATE_KEY_ORG}"
//! openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out "${TOKEN_ALGO_PRIVATE_KEY}"
//! openssl ec -in "${TOKEN_ALGO_PRIVATE_KEY_ORG}" -pubout -out "${TOKEN_ALGO_PUBLIC_KEY}"
//! ```
//!
//! ## S3
//!
//! ### Setting up AWS credentials
//!
//! <https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-envvars.html>
//!
//! ```bash
//! export AWS_ACCESS_KEY_ID=ACCESS_KEY
//! export AWS_SECRET_ACCESS_KEY=SECRET_KEY
//! ```
//!
//! ### S3 Upload a user data file (no file type restrictions + s3 archival)
//!
//! ```bash
//! export UPLOAD_FILE="./README.md"
//! export DATA_TYPE="file"
//! export S3_DATA_BUCKET="BUCKET_NAME"
//! export S3_DATA_PREFIX="user/data/file"
//! ```
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     -XPOST \
//!     --data-binary "@${UPLOAD_FILE}" \
//!     "https://0.0.0.0:3000/user/data" \
//!     -H "Bearer: ${TOKEN}" \
//!     -H 'user_id: 1' \
//!     -H 'comments: this is a test comment' \
//!     -H 'encoding: na' \
//!     -H 'Content-type: text/txt' \
//!     -H 'filename: README.md' \
//!     -H "data_type: ${DATA_TYPE}" | jq
//! ```
//!
//! ### Search user data (token must be for the POST-ed user id)
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user/data/search" \
//!     -XPOST \
//!     -H "Bearer: ${TOKEN}" \
//!     -d '{"user_id":1}' | jq
//! ```
//!
//! ### Update a single user data record (token must be for the PUT user id)
//!
//! ```bash
//! curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/user/data" \
//!     -XPUT \
//!     -H "Bearer: ${TOKEN}" \
//!     -d '{"user_id":1,"data_id":1,"comments":"updated comment using curl"}' | jq
//! ```
//!
//! ### Login and save the token as an env variable
//!
//! ```bash
//! export TOKEN=$(curl -s ${TLS_ARGS} \
//!     "https://0.0.0.0:3000/login" \
//!     -XPOST \
//!     -d '{"email":"user@email.com","password":"12345"}' | jq -r '.token')
//! ```
//!
//! ## Postgres DB
//!
//! ### View DB Tables
//!
//! #### Connect to postgres using tls
//!
//! ```bash
//! psql --set=sslmode=require -h 0.0.0.0 -p 5432 -U postgres -d mydb
//! ```
//!
//! #### Get public tables in the mydb
//!
//! ```bash
//! SELECT table_name FROM information_schema.tables WHERE table_schema='public';
//! ```
//!
//! ## Podman Image Push
//!
//! ```bash
//! cur_tag=$(cat Cargo.toml | grep version | head -1 | sed -e 's/"//g' | awk '{print $NF}')
//! podman push IMAGE_ID "docker://docker.io/jayjohnson/rust-restapi:${cur_tag}"
//! ```
//!
//! ## Helm Chart
//!
//! Please refer to the [Deploying the Rust Rest API helm chart into kubernetes guide](https://github.com/jay-johnson/restapi/blob/main/charts/rust-restapi/README.md) for deploying the example helm chart into a kubernetes cluster.
//!
//! ## Build Docs
//!
//! ```bash
//! cargo doc --example server
//! ```

extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate uuid;

// include files and sub directories
pub mod core;
pub mod handle_request;
pub mod is3;
pub mod jwt;
pub mod monitoring;
pub mod pools;
pub mod requests;
pub mod tls;
pub mod utils;
