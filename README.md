# Rust Rest API Stack with User Management, Kafka Message Publishing, S3 uploads/downloads, and Prometheus for Monitoring

A secure-by-default Rest API using [hyper](https://crates.io/crates/hyper), [tokio](https://crates.io/crates/tokio), [bb8](https://crates.io/crates/bb8), [kafka-threadpool](https://crates.io/crates/kafka-threadpool), postgres, and [prometheus](https://crates.io/crates/prometheus) for monitoring.

## Features

1.  User management and authentication stored in postgres
1.  Async s3 uploading and downloading (to/from local files or to/from memory)
1.  Decoupled, async kafka threadpool that uses environment variables to connect to a kafka cluster with client mtls for authentication and encryption in transit
1.  Async publishing for all successful user events to a kafka topic (topic default: ``user.events``) and partition key (key default: ``user-{user.id}``)
1.  Async kafka messaging for one-off messages using custom kafka topic(s), partition key(s) and custom header(s).

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
- Includes [pg4admin](https://www.pgadmin.org/docs/pgadmin4/latest/index.html) for database management in a browser (deployed with podman compose).

### TLS Encryption

- Includes a tls asset generator tool ([./tls/create-tls-assets.sh](https://github.com/jay-johnson/restapi/blob/main/tls/create-tls-assets.sh)) for building self-signed tls assets including your own private Certificate Authority (CA).

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

### Generate TLS Assets and a Private Certificate Authority (CA) using CFSSL

Generate new tls assets under the ``./tls`` directory with these commands:

```bash
cd tls
./create-tls-assets.sh
cd ..
```

Please refer to the [Generating TLS Assets with CFSSL](./tls/README.md) for more information.

### Generate JWT Private and Public Signing Keys

Generate new signing JWT keys under the ``./jwt`` directory with these commands:

```bash
cd jwt
./recreate-jwt.sh
cd ..
```

Please refer to the [How to build JWT private and public keys for the jsonwebtokens crate doc](./jwt/README.md) for more information.

### Deploy Postgres and pgAdmin using Podman

Please refer to the [Build and Deploy a Secured Postgres backend doc](./docker/db/README.md) for more information.

### Build API Server

```bash
cargo build --example server
```

### Run API Server

```bash
export RUST_BACKTRACE=1 && export RUST_LOG=info,kafka_threadpool=info && ./target/debug/examples/server
```

## Environment Variables

### Rest API

Environment Variable  | Default
--------------------- | -------
SERVER_NAME_API       | api
SERVER_NAME_LABEL     | rust-restapi
API_ENDPOINT          | 0.0.0.0:3000
API_TLS_DIR           | ./tls/api
API_TLS_CA            | ./tls/ca/ca.pem
API_TLS_CERT          | ./tls/api/server.pem
API_TLS_KEY           | ./tls/api/server-key.pem

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
POSTGRES_TLS_DIR      | ./tls/postgres
POSTGRES_TLS_CA       | ./tls/ca/ca.pem
POSTGRES_TLS_CERT     | ./tls/postgres/client.pem
POSTGRES_TLS_KEY      | ./tls/postgres/client-key.pem
POSTGRES_DB_CONN_TYPE | postgresql

### Kafka Cluster

Please refer to the [kafka_threadpool docs](https://crates.io/crates/kafka-threadpool) for more information.

Environment Variable             | Purpose / Value
-------------------------------- | ---------------
KAFKA_PUBLISH_EVENTS             | if set to ``true`` or ``1`` publish all user events to kafka
KAFKA_ENABLED                    | toggle the kafka_threadpool on with: ``true`` or ``1`` anything else disables the threadpool
KAFKA_LOG_LABEL                  | tracking label that shows up in all crate logs
KAFKA_BROKERS                    | comma-delimited list of brokers (``host1:port,host2:port,host3:port``)
KAFKA_TOPICS                     | comma-delimited list of supported topics
KAFKA_PUBLISH_RETRY_INTERVAL_SEC | number of seconds to sleep before each publish retry
KAFKA_PUBLISH_IDLE_INTERVAL_SEC  | number of seconds to sleep if there are no message to process
KAFKA_NUM_THREADS                | number of threads for the threadpool
KAFKA_TLS_CLIENT_KEY             | optional - path to the kafka mTLS key (./tls/kafka-cluster-0/client-key.pem)
KAFKA_TLS_CLIENT_CERT            | optional - path to the kafka mTLS certificate (./tls/kafka-cluster-0/client.pem)
KAFKA_TLS_CLIENT_CA              | optional - path to the kafka mTLS certificate authority (CA) (./tls/ca/ca.pem)
KAFKA_METADATA_COUNT_MSG_OFFSETS | optional - set to anything but ``true`` to bypass counting the offsets

#### Sample kafka.env file

```bash
# enable the cluster
export KAFKA_ENABLED=1
export KAFKA_LOG_LABEL="ktp"
export KAFKA_BROKERS="host1:port,host2:port,host3:port"
export KAFKA_TOPICS="testing"
export KAFKA_PUBLISH_RETRY_INTERVAL_SEC="1.0"
export KAFKA_NUM_THREADS="5"
export KAFKA_TLS_CLIENT_CA="./tls/ca/ca.pem"
export KAFKA_TLS_CLIENT_CERT="./tls/kafka-cluster-0/client.pem"
export KAFKA_TLS_CLIENT_KEY="./tls/kafka-cluster-0/client-key.pem"
# the KafkaPublisher can count the offsets for each topic with "true" or "1"
export KAFKA_METADATA_COUNT_MSG_OFFSETS="true"
```

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

This will build an initial base image using podman. Note: this base image will **not** work on a different cpu chipset because the openssl libraries are compiled within the image for this base image.

```bash
./build-base.sh
```

### Build Derived Image

By reusing the base image, this derived image only needs to recompile the server. With minimal code changes, this is a much faster build than the base image build.

```bash
./build-derived.sh
```

## Kubernetes

### Start Kafka

If you do not have a running Kafka cluster, you can deploy your own with:

https://github.com/jay-johnson/rust-with-strimzi-kafka-and-tls

### Helm Chart

#### Deploy TLS Assets into Kubernetes

This command will deploy all jwt keys, tls assets and credentials into the ``dev`` namespace:

```bash
./deploy-kubernetes-assets.sh -e dev
```

#### Deploy the Rust Rest API into Kubernetes

Please refer to the [Deploying the Rust Rest API helm chart into kubernetes guide](https://github.com/jay-johnson/restapi/blob/main/charts/rust-restapi/README.md) for deploying the example helm chart into a kubernetes cluster.

By default this uses the ``jayjohnson/rust-restapi`` container image

```bash
helm upgrade --install -n dev dev-api ./charts/rust-restapi -f ./charts/rust-restapi/values.yaml
```

## Monitoring

### Prometheus

This section assumes you have a working prometheus instance already running inside kubernetes. Below is the Prometheus ``scrape_config`` to monitor the rest api deployment replica(s) within kubernetes. Note this config also assumes the api chart is running in the ``dev`` namespace:

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
    - dev-api.dev.svc.cluster.local:3000
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

## Podman Image Push

```bash
cur_tag=$(cat Cargo.toml | grep version | head -1 | sed -e 's/"//g' | awk '{print $NF}')
podman tag IMAGE_ID "docker://docker.io/jayjohnson/rust-restapi:${cur_tag}"
podman tag IMAGE_ID "docker://docker.io/jayjohnson/rust-restapi:latest"
podman push "docker.io/jayjohnson/rust-restapi:${cur_tag}"
podman push "docker.io/jayjohnson/rust-restapi:latest"
```

## Build Docs

```bash
cargo doc --example server
```

## Start Development Server

```bash
source ./env/api.env && source ./env/kafka.env && source ./env/postgres.env && cargo build --example server && export RUST_BACKTRACE=1 && export RUST_LOG=info,kafka_threadpool=info,rdkafka=error && ./target/debug/examples/server
```
