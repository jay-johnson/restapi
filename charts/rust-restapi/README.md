# Rust Rest API with hyper, tokio, bb8 and prometheus for monitoring

## Install

### Deploy TLS Assets

```bash
./deploy-tls-assets.sh
```

### Install Chart

```bash
helm install -n default rust-restapi rust-restapi
```

## Environment Variables

The helm chart uses these default environment variables.

### Rest API

Environment Variable  | Default
--------------------- | -------
SERVER_NAME_API       | api
SERVER_NAME_LABEL     | rust-restapi
API_ENDPOINT          | 0.0.0.0:3000
API_TLS_DIR           | /server/certs/tls/api
API_TLS_CA            | /server/certs/tls/api/api-ca.pem
API_TLS_CERT          | /server/certs/tls/api/api.crt
API_TLS_KEY           | /server/certs/tls/api/api.key

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
POSTGRES_ENDPOINT     | postgres.default.svc.cluster.local:5432
POSTGRES_TLS_DIR      | /server/certs/tls/postgres
POSTGRES_TLS_CA       | /server/certs/tls/postgres/postgres-ca.pem
POSTGRES_TLS_CERT     | /server/certs/tls/postgres/postgres.crt
POSTGRES_TLS_KEY      | /server/certs/tls/postgres/postgres.key
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
TOKEN_ALGO_PRIVATE_KEY               | /server/certs/tls/jwt/private-key.pem
TOKEN_ALGO_PUBLIC_KEY                | /server/certs/tls/jwt/public-key.pem
SERVER_PKI_DIR_JWT                   | /server/certs/tls/jwt
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


#### Port Forward

```bash
kubectl port-forward -n default svc/rust-restapi 8080:3000 --address 0.0.0.0
```

## Uninstall Chart

```bash
helm delete -n default rust-restapi
```
