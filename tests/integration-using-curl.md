# Integration Tests Using curl Guide

## Set up bash curl tests

```bash
export API_TLS_DIR="./certs/tls/api"
export TLS_CERT_ARGS="--cacert ${API_TLS_DIR}/api-ca.pem \
    --cert ${API_TLS_DIR}/api.crt \
    --key ${API_TLS_DIR}/api.key"
```

## User APIs

### Login (user does not exist yet)

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/login" \
    -XPOST \
    -d '{"email":"user@email.com","password":"12345"}' | jq
```

### Create user

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user" \
    -XPOST \
    -d '{"email":"user@email.com","password":"12345"}' | jq
```

### Login and save the token as an env variable

```bash
export TOKEN=$(curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/login" \
    -XPOST \
    -d '{"email":"user@email.com","password":"12345"}' | jq -r '.token')
```

### Get user

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user/1" \
    -XGET \
    -H "Bearer: ${TOKEN}" | jq
```

### Update user

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user" \
    -H "Bearer: ${TOKEN}" \
    -XPUT \
    -d '{"user_id":1,"email":"somenewemail@gmail.com","password":"321123","state":0}'
```

### Change user password

#### Change to a new password

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user" \
    -H "Bearer: ${TOKEN}" \
    -XPUT \
    -d '{"user_id":1,"password":"12345a"}' | jq
```

#### Change password back to the original

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user" \
    -H "Bearer: ${TOKEN}" \
    -XPUT \
    -d '{"user_id":1,"password":"12345"}' | jq
```

### Create a one-time-use-password (otp) allowing a user to reset their users.password from the users.email

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user/password/reset" \
    -H "Bearer: ${TOKEN}" \
    -XPOST \
    -d '{"user_id":1,"email":"user@email.com"}' | jq
```

### Consume user one-time-use-password token to reset the users.password (otp)

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user/password/change" \
    -H "Bearer: ${TOKEN}" \
    -XPOST \
    -d '{"user_id":1,"email":"user@email.com"}' | jq
```

### Change user email

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user" \
    -H "Bearer: ${TOKEN}" \
    -XPUT \
    -d '{"user_id":1,"email":"unique@gmail.com"}' | jq
```

### Verify user email

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user/verify?u=1&t=2" | jq
```

### Search user (token must be for the POST-ed user id)

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user/search" \
    -XPOST \
    -H "Bearer: ${TOKEN}" \
    -d '{"email":"user","user_id":1}' | jq
```

### Delete user

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user" \
    -XDELETE \
    -d '{"email":"user@email.com","user_id":1}' \
    -H "Content-type: application/json" \
    -H "Bearer: ${TOKEN}" | jq
```

## JWT (json web tokens)

### Configurable JWT Environment Variables

#### Header key for the token:

```bash
export TOKEN_HEADER="Bearer"
```

#### Token Org (embedded in the jwt)

```bash
export TOKEN_ORG="Org Name";
```

#### Token Lifetime Duration

```bash
# 30 days
export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=2592000;
# 7 days
export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=604800;
# 1 day
export TOKEN_EXPIRATION_SECONDS_INTO_FUTURE=86400;
```

#### Token Encryption Keys

```bash
export TOKEN_ALGO_KEY_DIR="./jwt"
export TOKEN_ALGO_PRIVATE_KEY_ORG="${TOKEN_ALGO_KEY_DIR}/private-key.pem"
export TOKEN_ALGO_PRIVATE_KEY="${TOKEN_ALGO_KEY_DIR}/private-key-pkcs8.pem"
export TOKEN_ALGO_PUBLIC_KEY="${TOKEN_ALGO_KEY_DIR}/public-key.pem"
```

##### Generate your own jwt keys with these commands

These commands were tested on ubuntu 21.10 using bash:

```bash
openssl ecparam -name prime256v1 -genkey -out "${TOKEN_ALGO_PRIVATE_KEY_ORG}"
openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out "${TOKEN_ALGO_PRIVATE_KEY}"
openssl ec -in "${TOKEN_ALGO_PRIVATE_KEY_ORG}" -pubout -out "${TOKEN_ALGO_PUBLIC_KEY}"
```

## S3

### Setting up AWS credentials

https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-envvars.html

```bash
export AWS_ACCESS_KEY_ID=ACCESS_KEY
export AWS_SECRET_ACCESS_KEY=SECRET_KEY
```

### S3 Upload a user data file (no file type restrictions + s3 archival)

```bash
export UPLOAD_FILE="./README.md"
export DATA_TYPE="file"
export S3_DATA_BUCKET="BUCKET_NAME"
export S3_DATA_PREFIX="user/data/file"
```

```bash
curl -s ${TLS_CERT_ARGS} \
    -XPOST \
    --data-binary "@${UPLOAD_FILE}" \
    "https://0.0.0.0:3000/user/data" \
    -H "Bearer: ${TOKEN}" \
    -H 'user_id: 1' \
    -H 'comments: this is a test comment' \
    -H 'encoding: na' \
    -H 'Content-type: text/txt' \
    -H 'filename: README.md' \
    -H "data_type: ${DATA_TYPE}" | jq
```

### Search user data (token must be for the POST-ed user id)

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user/data/search" \
    -XPOST \
    -H "Bearer: ${TOKEN}" \
    -d '{"user_id":1}' | jq
```

### Update a single user data record (token must be for the PUT user id)

```bash
curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/user/data" \
    -XPUT \
    -H "Bearer: ${TOKEN}" \
    -d '{"user_id":1,"data_id":1,"comments":"updated comment using curl"}' | jq
```

### Login and save the token as an env variable

```bash
export TOKEN=$(curl -s ${TLS_CERT_ARGS} \
    "https://0.0.0.0:3000/login" \
    -XPOST \
    -d '{"email":"user@email.com","password":"12345"}' | jq -r '.token')
```

## Postgres DB

### View DB Tables

#### Connect to postgres using tls

```bash
psql --set=sslmode=require -h 0.0.0.0 -p 5432 -U postgres -d mydb
```

#### Get public tables in the mydb

```bash
SELECT table_name FROM information_schema.tables WHERE table_schema='public';
```
