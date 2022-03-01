## Rust Rest API Stack with User Management

A secure-by-default rest api stack implemented with hyper, tokio and postgres.

This project is focused on providing end-to-end encryption by default for 12-factor applications looking to customize functionality using environment variables as needed.

Comes with a working user management and authentication backend written in postgresql.

For ease of use, you can browse the database using pg4admin for database management (deployed with docker compose).

### Overview

- User authentication enabled by default and implemented with custom tls assets to encrypt all JWT tokens with storage in postgres.
- Users can upload and manage files stored on AWS S3 (assuming valid credentials are loaded outside this rust project).
- User password reset and user email change support using one-time-use tokens that are stored in postgres.
- User passwords are salted using argon2
- The hyper server hosts tls assets that can be re-generated with the tools in this repository.
- The postgres database requires clients present the included tls Certificate Authority file for encrypting data in-transit.
- The rest api server accesses postgres with a bb8 client threadpool.
- encrypted JWT keys included and documentation to build new ones as you need.

### TLS Encryption Status

Compoment        | Status
---------------- | ------
Rest API Server  | Listening for encrypted client connections on tcp port **3000**
JWT              | Encrypting and decrypting tokens with [ECDSA using SHA-256](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/enum.Algorithm.html#variant.ES256)
Postgres         | Listening for encrypted client connections on tcp port **5432** (tls Certificate Authority required)
pgAdmin          | Listening for encrypted HTTP client connections on tcp port **5433**
AWS S3           | Encrypted at rest with [AES256](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard)

## Getting Started

### Clone the repo

```bash
git clone https://github.com/jay-johnson/restapi
cd restapi
```

### Generate TLS Assets

The repository [restapi](https://github.com/jay-johnson/restapi) includes default tls assets, but for security purposes you should generate your own. Please refer to the [Generate TLS Assets doc](./certs/README.md) for more information.

Here's how to generate them under the ``./certs`` directory:

<a href="https://asciinema.org/a/473131?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473131.png"/></a>

```bash
cd certs
./generate-tls-assets.sh -f -c ./configs/dev-network.yml
cd ..
```

### Generate JWT Keys

Authentication using JWT requires encrypting and decrypting using your own keys. Please refer to the [How to build JWT private and public keys for the jsonwebtokens crate doc](./certs/README.md) for more information.

Here's how to generate the jwt keys under the ``./jwt`` directory:

<a href="https://asciinema.org/a/473132?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473132.png"/></a>

```bash
cd jwt
openssl ecparam -name prime256v1 -genkey -out private-key.pem
openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out private-key-pkcs8.pem
openssl ec -in private-key.pem -pubout -out public-key.pem
cd ..
```

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

## Supported APIs

Here are the supported json contracts for each ``Request`` and ``Response`` based off the url. Each client request is handled by the [./src/handle_requests.rs module](./src/handle_request.rs) and returned as a response back to the client (serialization using ``serde_json``)

### User APIs

#### Create User

Create a single ``users`` record for the new user

- URL path: ``/user``
- Method: ``POST``
- Handler: [create_user](https://docs.rs/restapi/1.0.2/restapi/requests/user/create_user/fn.create_user.html)
- Request: [ApiReqUserCreate](https://docs.rs/restapi/1.0.2/restapi/requests/user/create_user/struct.ApiReqUserCreate.html)
- Response: [ApiResUserCreate](https://docs.rs/restapi/1.0.2/restapi/requests/user/create_user/struct.ApiResqUserCreate.html)

#### Update User

Update supported ``users`` fields (including change user email and password)

- URL path: ``/user``
- Method: ``PUT``
- Handler: [update_user](https://docs.rs/restapi/1.0.2/restapi/requests/user/update_user/fn.update_user.html)
- Request: [ApiReqUserUpdate](https://docs.rs/restapi/1.0.2/restapi/requests/user/update_user/struct.ApiReqUserUpdate.html)
- Response: [ApiResUserUpdate](https://docs.rs/restapi/1.0.2/restapi/requests/user/update_user/struct.ApiResqUserUpdate.html)

#### Get User

Get a single user by ``users.id`` - by default, a user can only get their own account details

- URL path: ``/user/USERID``
- Method: ``GET``
- Handler: [get_user](https://docs.rs/restapi/1.0.2/restapi/requests/user/get_user/fn.get_user.html)
- Request: [ApiReqUserGet](https://docs.rs/restapi/1.0.2/restapi/requests/user/get_user/struct.ApiReqUserGet.html)
- Response: [ApiResUserGet](https://docs.rs/restapi/1.0.2/restapi/requests/user/get_user/struct.ApiResqUserGet.html)

#### Delete User

Delete a single ``users`` record (note: this does not delete the db record, just sets the ``users.state`` to inactive ``1``)

- URL path: ``/user``
- Method: ``DELETE``
- Handler: [delete_user](https://docs.rs/restapi/1.0.2/restapi/requests/user/delete_user/fn.delete_user.html)
- Request: [ApiReqUserDelete](https://docs.rs/restapi/1.0.2/restapi/requests/user/delete_user/struct.ApiReqUserDelete.html)
- Response: [ApiResUserDelete](https://docs.rs/restapi/1.0.2/restapi/requests/user/delete_user/struct.ApiResqUserDelete.html)

#### Search Users in the db

Search for matching ``users`` records in the db

- URL path: ``/user/search``
- Method: ``POST``
- Handler: [search_users](https://docs.rs/restapi/1.0.2/restapi/requests/user/search_users/fn.search_users.html)
- Request: [ApiReqUserSearch](https://docs.rs/restapi/1.0.2/restapi/requests/user/search_users/struct.ApiReqUserSearch.html)
- Response: [ApiResUserSearch](https://docs.rs/restapi/1.0.2/restapi/requests/user/search_users/struct.ApiResqUserSearch.html)

#### Create One-Time-Use Password Reset Token (OTP)

Create a one-time-use password reset token that allows a user to change their ``users.password`` value by presenting the token

- URL path: ``/user/password/reset``
- Method: ``POST``
- Handler: [create_otp](https://docs.rs/restapi/1.0.2/restapi/requests/user/create_otp/fn.create_otp.html)
- Request: [ApiReqUserCreateOtp](https://docs.rs/restapi/1.0.2/restapi/requests/user/consume_user_otp/struct.ApiReqUserCreateOtp.html)
- Response: [ApiResUserCreateOtp](https://docs.rs/restapi/1.0.2/restapi/requests/user/consume_user_otp/struct.ApiResUserCreateOtp.html)

#### Consume a One-Time-Use Password Reset Token (OTP)

Consume a one-time-use password and change the user's ``users.password`` value to the new argon2-salted password

- URL path: ``/user/password/change``
- Method: ``POST``
- Handler: [consume_user_otp](https://docs.rs/restapi/1.0.2/restapi/requests/user/consume_user_otp/fn.consume_user_otp.html)
- Request: [ApiReqUserConsumeOtp](https://docs.rs/restapi/1.0.2/restapi/requests/user/create_otp/struct.ApiReqUserConsumeOtp.html)
- Response: [ApiResUserConsumeOtp](https://docs.rs/restapi/1.0.2/restapi/requests/user/create_otp/struct.ApiResUserConsumeOtp.html)

#### Verify a User's email

Consume a one-time-use verification token and change the user's ``users.verified`` value verified (``1``)

- URL path: ``/user/verify``
- Method: ``GET``
- Handler: [verify_user](https://docs.rs/restapi/1.0.2/restapi/requests/user/verify_user/fn.verify_user.html)
- Request: [ApiReqUserVerify](https://docs.rs/restapi/1.0.2/restapi/requests/user/verify_user/struct.ApiReqUserVerify.html)
- Response: [ApiResUserVerify](https://docs.rs/restapi/1.0.2/restapi/requests/user/verify_user/struct.ApiResUserVerify.html)

### User S3 APIs

#### Upload a file asynchronously to AWS S3 and store a tracking record in the db

Upload a local file on disk to AWS S3 asynchronously and store a tracking record in the ``users_data`` table. The documentation refers to this as a ``user data`` or ``user data file`` record.

- URL path: ``/user/data``
- Method: ``POST``
- Handler: [upload_user_data](https://docs.rs/restapi/1.0.2/restapi/requests/user/upload_user_data/fn.upload_user_data.html)
- Request: [ApiReqUserUploadData](https://docs.rs/restapi/1.0.2/restapi/requests/user/upload_user_data/struct.ApiReqUserUploadData.html)
- Response: [ApiResUserUploadData](https://docs.rs/restapi/1.0.2/restapi/requests/user/upload_user_data/struct.ApiResUserUploadData.html)

#### Update an existing user data file record for a file stored in AWS S3

Update the ``users_data`` tracking record for a file that exists in AWS S3

- URL path: ``/user/data``
- Method: ``PUT``
- Handler: [update_user_data](https://docs.rs/restapi/1.0.2/restapi/requests/user/update_user_data/fn.update_user_data.html)
- Request: [ApiReqUserUpdateData](https://docs.rs/restapi/1.0.2/restapi/requests/user/update_user_data/struct.ApiReqUserUpdateData.html)
- Response: [ApiResUserUpdateData](https://docs.rs/restapi/1.0.2/restapi/requests/user/update_user_data/struct.ApiResUserUpdateData.html)

#### Search for existing user data files from the db

Search for matching records in the ``users_data`` db based off the request's values

- URL path: ``/user/data/search``
- Method: ``POST``
- Handler: [search_user_data](https://docs.rs/restapi/1.0.2/restapi/requests/user/search_user_data/fn.search_user_data.html)
- Request: [ApiReqUserSearchData](https://docs.rs/restapi/1.0.2/restapi/requests/user/search_user_data/struct.ApiReqUserSearchData.html)
- Response: [ApiResUserSearchData](https://docs.rs/restapi/1.0.2/restapi/requests/user/search_user_data/struct.ApiResUserSearchData.html)

### User Authentication APIs

#### User Login

Log the user in and get a json web token (jwt) back for authentication on subsequent client requests

- URL path: ``/login``
- Method: ``POST``
- Handler: [login](https://docs.rs/restapi/1.0.2/restapi/requests/auth/login_user/fn.login_user.html)
- Request: [ApiReqUserLogin](https://docs.rs/restapi/1.0.2/restapi/requests/auth/login_user/struct.ApiReqUserLogin.html)
- Response: [ApiResUserLogin](https://docs.rs/restapi/1.0.2/restapi/requests/auth/login_user/struct.ApiResUserLogin.html)

## Integration Tests

This project focused on integration tests for v1 instead of only rust tests (specifically everything has been tested with **curl**):

Please refer to the [Integration Tests Using curl Guide](./tests/integration-using-curl.md)

