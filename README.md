# Rust Rest API Stack with User Management and Prometheus for Monitoring

A secure-by-default rest api stack implemented with hyper, tokio, bb8 and postgres with prometheus for monitoring. This project is focused on providing end-to-end encryption by default for 12-factor applications looking to customize functionality using environment variables as needed. Includes a working user management and authentication backend written in postgresql with async S3 uploading for POST-ed data files.

## Overview

### User

- User password reset and user email change support using one-time-use tokens that are stored in postgres.
- Users can upload and manage files stored on AWS S3 (assuming valid credentials are loaded outside this rust project).
- User passwords are hashed using [argon2](https://docs.rs/argon2/latest/argon2/).

### Auth

- User authentication enabled by default
- Default JWT signing keys included with [documentation for building new keys as needed](https://github.com/jay-johnson/restapi/tree/main/jwt).

### Database

- The rest api server utilizes postgres with a [bb8 client threadpool](https://github.com/djc/bb8).
- The postgres database requires each client connection include the postgres tls certificate authority file for encrypting data in-transit.
- Includes [pg4admin](https://www.pgadmin.org/docs/pgadmin4/latest/index.html) for database management in a browser (deployed with docker compose).

### TLS Encryption

- Includes a tls asset generator tool ([./certs/generate-tls-assets.sh](https://github.com/jay-johnson/restapi/blob/main/certs/generate-tls-assets.sh)) for building self-signed tls assets (requires docker).

#### Ingress

Component        | Status
---------------- | ------
Rest API Server  | Listening for encrypted client connections on tcp port **3000**
Postgres         | Listening for encrypted client connections on tcp port **5432** (tls Certificate Authority required)
pgAdmin          | Listening for encrypted HTTP client connections on tcp port **5433**

## Getting Started

### Clone the repo

```bash
git clone https://github.com/jay-johnson/restapi
cd restapi
```

### Generate TLS Assets

The repository [restapi](https://github.com/jay-johnson/restapi) includes default tls assets, but for security purposes you should generate your own. Please refer to the [Generate TLS Assets doc](./certs/README.md) for more information.

Here's how to generate them under the ``./certs`` directory:

```bash
cd certs
./generate-tls-assets.sh -f -c ./configs/dev-network.yml
cd ..
```

<a href="https://asciinema.org/a/473131?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473131.png"/></a>

### Generate JWT Keys

This repo includes default JWT signing keys, but you should generate your own signing keys under the ``./jwt`` directory with these commands:

```bash
cd jwt
openssl ecparam -name prime256v1 -genkey -out private-key.pem
openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out private-key-pkcs8.pem
openssl ec -in private-key.pem -pubout -out public-key.pem
cd ..
```

<a href="https://asciinema.org/a/473132?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473132.png"/></a>

Please refer to the [How to build JWT private and public keys for the jsonwebtokens crate doc](./certs/README.md) for more information.

### Build the Postgres docker image

Please refer to the [Build and Deploy a Secured Postgres backend doc](./docker/db/README.md) for more information.

<a href="https://asciinema.org/a/473134?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473134.png"/></a>

### Build API Server

```bash
cargo build --example server
```

### Run API Server

```bash
export RUST_BACKTRACE=1 && export RUST_LOG=info && ./target/debug/examples/server
```

## Environment Variables

### Rest API

Environment Variable  | Default
--------------------- | -------
SERVER_NAME_API       | api
SERVER_NAME_LABEL     | rust-restapi
API_ENDPOINT          | 0.0.0.0:3000
API_TLS_DIR           | ./certs/tls/api
API_TLS_CA            | ./certs/tls/api/api-ca.pem
API_TLS_CERT          | ./certs/tls/api/api.crt
API_TLS_KEY           | ./certs/tls/api/api.key

### User Email Verification

Environment Variable                   | Default
-------------------------------------- | -------
USER_EMAIL_VERIFICATION_REQUIRED       | "0"
USER_EMAIL_VERIFICATION_ENABLED        | "1"
USER_EMAIL_VERIFICATION_EXP_IN_SECONDS | "2592000"

### User One-Time-Use Token Expiration for Password Recovery

Environment Variable    | Default
----------------------- | -------
USER_OTP_EXP_IN_SECONDS | "2592000"

### Postgres Database

Environment Variable  | Default
--------------------- | -------
POSTGRES_USERNAME     | datawriter
POSTGRES_PASSWORD     | "123321"
POSTGRES_ENDPOINT     | 0.0.0.0:5432
POSTGRES_TLS_DIR      | ./certs/tls/postgres
POSTGRES_TLS_CA       | ./certs/tls/postgres/postgres-ca.pem
POSTGRES_TLS_CERT     | ./certs/tls/postgres/postgres.crt
POSTGRES_TLS_KEY      | ./certs/tls/postgres/postgres.key
POSTGRES_DB_CONN_TYPE | postgresql

### S3

Environment Variable | Default
-------------------- | -------
S3_DATA_BUCKET       | YOUR_BUCKET
S3_DATA_PREFIX       | /rust-restapi/tests
S3_STORAGE_CLASS     | STANDARD
S3_DATA_UPLOAD_TO_S3 | "0"

### JWT

Environment Variable                 | Default
------------------------------------ | -------
TOKEN_EXPIRATION_SECONDS_INTO_FUTURE | "2592000"
TOKEN_ORG                            | example.org
TOKEN_HEADER                         | Bearer
TOKEN_ALGO_PRIVATE_KEY               | ./jwt/private-key-pkcs8.pem
TOKEN_ALGO_PUBLIC_KEY                | ./jwt/public-key.pem
SERVER_PKI_DIR_JWT                   | ./jwt
SERVER_PASSWORD_SALT                 | 78197b60-c950-4339-a52c-053165a04764

### Rust

Environment Variable | Default
-------------------- | -------
RUST_BACKTRACE       | "1"
RUST_LOG             | info

### Debug

Environment Variable | Default
-------------------- | -------
DEBUG                | "1"

## Docker Builds

### Build Base Image

This will build an initial base image inside a docker container. Note: this base image will **not** work on a different cpu chipset because the openssl libraries are compiled within the image for this base image.

```bash
./build-base.sh
```

### Build Derived Image

By reusing the base image, this derived image only needs to recompile the server. With minimal code changes, this is a much faster build than the base image build.

```bash
./build-derived.sh
```

## Kubernetes

### Helm Chart

#### Deploy TLS Assets into Kubernetes

```bash
./deploy-tls-assets.sh
```

#### Deploy the Rust Rest API into Kubernetes

By default this uses ``jayjohnson/rust-restapi`` image by default

```bash
helm install -n default rust-restapi ./charts/rust-restapi
```

## Monitoring

### Prometheus

This section assumes you have a working prometheus instance already running inside kubernetes. Below is the Prometheus ``scrape_config`` to monitor the rest api deployment replica(s) within kubernetes. Note this config also assumes the api chart is running in the ``default`` namespace:

```yaml
scrape_configs:
- job_name: rust-restapi
  scrape_interval: 10s
  scrape_timeout: 5s
  metrics_path: /metrics
  scheme: https
  tls_config:
    insecure_skip_verify: true
  static_configs:
  - targets:
    - rust-restapi.default.svc.cluster.local:3000
```

## Supported APIs

Here are the supported json contracts for each ``Request`` and ``Response`` based off the url. Each client request is handled by the [./src/handle_requests.rs module](./src/handle_request.rs) and returned as a response back to the client (serialization using ``serde_json``)

### User APIs

#### Create User

Create a single ``users`` record for the new user

- URL path: ``/user``
- Method: ``POST``
- Handler: [create_user](https://docs.rs/restapi/latest/restapi/requests/user/create_user/fn.create_user.html)
- Request: [ApiReqUserCreate](https://docs.rs/restapi/latest/restapi/requests/user/create_user/struct.ApiReqUserCreate.html)
- Response: [ApiResUserCreate](https://docs.rs/restapi/latest/restapi/requests/user/create_user/struct.ApiResqUserCreate.html)

#### Update User

Update supported ``users`` fields (including change user email and password)

- URL path: ``/user``
- Method: ``PUT``
- Handler: [update_user](https://docs.rs/restapi/latest/restapi/requests/user/update_user/fn.update_user.html)
- Request: [ApiReqUserUpdate](https://docs.rs/restapi/latest/restapi/requests/user/update_user/struct.ApiReqUserUpdate.html)
- Response: [ApiResUserUpdate](https://docs.rs/restapi/latest/restapi/requests/user/update_user/struct.ApiResqUserUpdate.html)

#### Get User

Get a single user by ``users.id`` - by default, a user can only get their own account details

- URL path: ``/user/USERID``
- Method: ``GET``
- Handler: [get_user](https://docs.rs/restapi/latest/restapi/requests/user/get_user/fn.get_user.html)
- Request: [ApiReqUserGet](https://docs.rs/restapi/latest/restapi/requests/user/get_user/struct.ApiReqUserGet.html)
- Response: [ApiResUserGet](https://docs.rs/restapi/latest/restapi/requests/user/get_user/struct.ApiResqUserGet.html)

#### Delete User

Delete a single ``users`` record (note: this does not delete the db record, just sets the ``users.state`` to inactive ``1``)

- URL path: ``/user``
- Method: ``DELETE``
- Handler: [delete_user](https://docs.rs/restapi/latest/restapi/requests/user/delete_user/fn.delete_user.html)
- Request: [ApiReqUserDelete](https://docs.rs/restapi/latest/restapi/requests/user/delete_user/struct.ApiReqUserDelete.html)
- Response: [ApiResUserDelete](https://docs.rs/restapi/latest/restapi/requests/user/delete_user/struct.ApiResqUserDelete.html)

#### Search Users in the db

Search for matching ``users`` records in the db

- URL path: ``/user/search``
- Method: ``POST``
- Handler: [search_users](https://docs.rs/restapi/latest/restapi/requests/user/search_users/fn.search_users.html)
- Request: [ApiReqUserSearch](https://docs.rs/restapi/latest/restapi/requests/user/search_users/struct.ApiReqUserSearch.html)
- Response: [ApiResUserSearch](https://docs.rs/restapi/latest/restapi/requests/user/search_users/struct.ApiResqUserSearch.html)

#### Create One-Time-Use Password Reset Token (OTP)

Create a one-time-use password reset token that allows a user to change their ``users.password`` value by presenting the token

- URL path: ``/user/password/reset``
- Method: ``POST``
- Handler: [create_otp](https://docs.rs/restapi/latest/restapi/requests/user/create_otp/fn.create_otp.html)
- Request: [ApiReqUserCreateOtp](https://docs.rs/restapi/latest/restapi/requests/user/create_otp/struct.ApiReqUserCreateOtp.html)
- Response: [ApiResUserCreateOtp](https://docs.rs/restapi/latest/restapi/requests/user/create_otp/struct.ApiResUserCreateOtp.html)

#### Consume a One-Time-Use Password Reset Token (OTP)

Consume a one-time-use password and change the user's ``users.password`` value to the new argon2-hashed password

- URL path: ``/user/password/change``
- Method: ``POST``
- Handler: [consume_user_otp](https://docs.rs/restapi/latest/restapi/requests/user/consume_user_otp/fn.consume_user_otp.html)
- Request: [ApiReqUserConsumeOtp](https://docs.rs/restapi/latest/restapi/requests/user/consume_user_otp/struct.ApiReqUserConsumeOtp.html)
- Response: [ApiResUserConsumeOtp](https://docs.rs/restapi/latest/restapi/requests/user/consume_user_otp/struct.ApiResUserConsumeOtp.html)

#### Verify a User's email

Consume a one-time-use verification token and change the user's ``users.verified`` value verified (``1``)

- URL path: ``/user/verify``
- Method: ``GET``
- Handler: [verify_user](https://docs.rs/restapi/latest/restapi/requests/user/verify_user/fn.verify_user.html)
- Request: [ApiReqUserVerify](https://docs.rs/restapi/latest/restapi/requests/user/verify_user/struct.ApiReqUserVerify.html)
- Response: [ApiResUserVerify](https://docs.rs/restapi/latest/restapi/requests/user/verify_user/struct.ApiResUserVerify.html)

### User S3 APIs

#### Upload a file asynchronously to AWS S3 and store a tracking record in the db

Upload a local file on disk to AWS S3 asynchronously and store a tracking record in the ``users_data`` table. The documentation refers to this as a ``user data`` or ``user data file`` record.

- URL path: ``/user/data``
- Method: ``POST``
- Handler: [upload_user_data](https://docs.rs/restapi/latest/restapi/requests/user/upload_user_data/fn.upload_user_data.html)
- Request: [ApiReqUserUploadData](https://docs.rs/restapi/latest/restapi/requests/user/upload_user_data/struct.ApiReqUserUploadData.html)
- Response: [ApiResUserUploadData](https://docs.rs/restapi/latest/restapi/requests/user/upload_user_data/struct.ApiResUserUploadData.html)

#### Update an existing user data file record for a file stored in AWS S3

Update the ``users_data`` tracking record for a file that exists in AWS S3

- URL path: ``/user/data``
- Method: ``PUT``
- Handler: [update_user_data](https://docs.rs/restapi/latest/restapi/requests/user/update_user_data/fn.update_user_data.html)
- Request: [ApiReqUserUpdateData](https://docs.rs/restapi/latest/restapi/requests/user/update_user_data/struct.ApiReqUserUpdateData.html)
- Response: [ApiResUserUpdateData](https://docs.rs/restapi/latest/restapi/requests/user/update_user_data/struct.ApiResUserUpdateData.html)

#### Search for existing user data files from the db

Search for matching records in the ``users_data`` db based off the request's values

- URL path: ``/user/data/search``
- Method: ``POST``
- Handler: [search_user_data](https://docs.rs/restapi/latest/restapi/requests/user/search_user_data/fn.search_user_data.html)
- Request: [ApiReqUserSearchData](https://docs.rs/restapi/latest/restapi/requests/user/search_user_data/struct.ApiReqUserSearchData.html)
- Response: [ApiResUserSearchData](https://docs.rs/restapi/latest/restapi/requests/user/search_user_data/struct.ApiResUserSearchData.html)

### User Authentication APIs

#### User Login

Log the user in and get a json web token (jwt) back for authentication on subsequent client requests

- URL path: ``/login``
- Method: ``POST``
- Handler: [login](https://docs.rs/restapi/latest/restapi/requests/auth/login_user/fn.login_user.html)
- Request: [ApiReqUserLogin](https://docs.rs/restapi/latest/restapi/requests/auth/login_user/struct.ApiReqUserLogin.html)
- Response: [ApiResUserLogin](https://docs.rs/restapi/latest/restapi/requests/auth/login_user/struct.ApiResUserLogin.html)

## Integration Tests

This project focused on integration tests for v1 instead of only rust tests (specifically everything has been tested with **curl**):

Please refer to the [Integration Tests Using curl Guide](./tests/integration-using-curl.md)

## Build Docs

```bash
cargo doc --example server
```
