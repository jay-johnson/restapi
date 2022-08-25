#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

yellow "deploying tls assets and credential secrets into kubernetes"

# set these to your values:
export S3_DATA_ACCESS_KEY="AWS_ACCESS_KEY_ID"
export S3_DATA_SECRET_KEY="AWS_SECRET_ACCESS_KEY"

export DB_POSTGRES_USERNAME="postgres"
export DB_POSTGRES_PASSWORD="postgres"

export DB_API_USERNAME="datawriter"
export DB_API_PASSWORD="123321"

export NAMESPACE="default"
export ENV_NAME="dev"
export APP_NAME="api"

if [[ "${1}" != "" ]]; then
    export ENV_NAME="${1}"
fi
if [[ "${2}" != "" ]]; then
    export APP_NAME="${2}"
fi

if [[ "${ENV_NAME}" == "" ]]; then
    red "missing env name as 1st arg"
    exit 1
fi

if [[ "${APP_NAME}" == "" ]]; then
    red "missing app name as 2nd arg"
    exit 1
fi

export BASE_PATH_TO_CERTS="../certs"
export BASE_PATH_TO_JWT="../jwt"

function deploy_rust_rest_api_secrets_jwt() {
    var_path_to_jwt_key_private="${BASE_PATH_TO_JWT}/private-key-pkcs8.pem"
    var_path_to_jwt_key_public="${BASE_PATH_TO_JWT}/public-key.pem"
    kubectl delete -n "${NAMESPACE}" secret "${SECRET_PREFIX}-api-jwt-keys" --ignore-not-found
    var_command="kubectl create -n ${NAMESPACE} secret generic ${SECRET_PREFIX}-api-jwt-keys --from-file=private-key.pem=${var_path_to_jwt_key_private} --from-file=public-key.pem=${var_path_to_jwt_key_public}"
    eval "${var_command}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api jwt keys as a secret - stopping"
        echo -e "\n${var_command}\n"
        exit 1
    fi
} # deploy_rust_rest_api_secrets_jwt - end

function deploy_rust_rest_api_secrets_tls() {
    var_path_to_api_tls_ca="${BASE_PATH_TO_CERTS}/tls/api/api-ca.pem"
    var_path_to_api_tls_crt="${BASE_PATH_TO_CERTS}/tls/api/api.crt"
    var_path_to_api_tls_key="${BASE_PATH_TO_CERTS}/tls/api/api.key"
    kubectl delete -n "${NAMESPACE}" secret "${SECRET_PREFIX}-api-tls" --ignore-not-found
    var_command="kubectl create -n ${NAMESPACE} secret generic ${SECRET_PREFIX}-api-tls --from-file=api.crt=${var_path_to_api_tls_crt} --from-file=api.key=${var_path_to_api_tls_key} --from-file=api-ca.pem=${var_path_to_api_tls_ca}"
    eval "${var_command}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api server tls secret - stopping"
        echo -e "\n${var_command}\n"
        exit 1
    fi
} # deploy_rust_rest_api_secrets_tls - end

function deploy_rust_rest_db_secrets_tls() {
    var_path_to_db_tls_ca="${BASE_PATH_TO_CERTS}/tls/postgres/postgres-ca.pem"
    var_path_to_db_tls_crt="${BASE_PATH_TO_CERTS}/tls/postgres/postgres.crt"
    var_path_to_db_tls_key="${BASE_PATH_TO_CERTS}/tls/postgres/postgres.key"
    kubectl delete -n "${NAMESPACE}" secret "${SECRET_PREFIX}-postgres-tls" --ignore-not-found
    var_command="kubectl create -n ${NAMESPACE} secret generic ${SECRET_PREFIX}-postgres-tls --from-file=postgres.crt=${var_path_to_db_tls_crt} --from-file=postgres.key=${var_path_to_db_tls_key} --from-file=postgres-ca.pem=${var_path_to_db_tls_ca}"
    eval "${var_command}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api database tls secret - stopping"
        echo -e "\n${var_command}\n"
        exit 1
    fi
} # deploy_rust_rest_api_secrets_tls - end

function deploy_rust_rest_pgadmin_secrets_tls() {
    var_path_to_pgadmin_tls_ca="${BASE_PATH_TO_CERTS}/tls/pgadmin/pgadmin-ca.pem"
    var_path_to_pgadmin_tls_crt="${BASE_PATH_TO_CERTS}/tls/pgadmin/pgadmin.crt"
    var_path_to_pgadmin_tls_key="${BASE_PATH_TO_CERTS}/tls/pgadmin/pgadmin.key"
    kubectl delete -n "${NAMESPACE}" secret "${SECRET_PREFIX}-pgadmin-tls" --ignore-not-found
    # https://www.pgadmin.org/docs/pgadmin4/6.7/container_deployment.html#mapped-files-and-directories
    var_command="kubectl create -n ${NAMESPACE} secret generic ${SECRET_PREFIX}-pgadmin-tls --from-file=server.cert=${var_path_to_pgadmin_tls_crt} --from-file=server.key=${var_path_to_pgadmin_tls_key} --from-file=pgadmin-ca.pem=${var_path_to_pgadmin_tls_ca}"
    eval "${var_command}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api pgadmin tls secret - stopping"
        echo -e "\n${var_command}\n"
        exit 1
    fi
} # deploy_rust_rest_pgadmin_secrets_tls - end

function deploy_s3_write_credentials() {
    var_secret_name="${ENV_NAME}-${APP_NAME}-s3-credentials"
    kubectl delete -n "${NAMESPACE}" secret "${var_secret_name}" --ignore-not-found
    var_s3_ak="${S3_DATA_ACCESS_KEY}"
    var_s3_sk="${S3_DATA_SECRET_KEY}"
    var_last_status="$?"
    kubectl create secret -n "${NAMESPACE}" generic "${var_secret_name}" --from-literal=access-key="${var_s3_ak}" --from-literal=secret-key="${var_s3_sk}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "failed to create ${var_secret_name} for env: ${ENV_NAME} - stopping"
        exit 1
    fi
} # deploy_s3_write_credentials - end

function deploy_postgres_db_credentials() {
    var_secret_name="${ENV_NAME}-postgres-db-credentials"
    kubectl delete -n "${NAMESPACE}" secret "${var_secret_name}" --ignore-not-found
    kubectl create secret -n "${NAMESPACE}" generic "${var_secret_name}" --from-literal=username="${DB_POSTGRES_USERNAME}" --from-literal=password="${DB_POSTGRES_PASSWORD}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "failed to create ${var_secret_name} for env: ${ENV_NAME} - stopping"
        exit 1
    fi
} # deploy_postgres_db_credentials - end

function deploy_api_db_credentials() {
    var_secret_name="${ENV_NAME}-${APP_NAME}-db-credentials"
    kubectl delete -n "${NAMESPACE}" secret "${var_secret_name}" --ignore-not-found
    kubectl create secret -n "${NAMESPACE}" generic "${var_secret_name}" --from-literal=username="${DB_API_USERNAME}" --from-literal=password="${DB_API_PASSWORD}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "failed to create ${var_secret_name} for env: ${ENV_NAME} - stopping"
        exit 1
    fi
} # deploy_api_db_credentials - end

function deploy_tls_secrets() {
    export SECRET_PREFIX="${ENV_NAME}"
    yellow "${ENV_NAME} - ${APP_NAME} - deploying tls cert_base_dir=${BASE_PATH_TO_CERTS} and jwt=${BASE_PATH_TO_JWT}"
    kubectl get ns "${NAMESPACE}" > /dev/null 2>&1
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        kubectl create ns "${NAMESPACE}"
        var_last_status="$?"
        if [[ "${var_last_status}" -ne 0 ]]; then
            red "failed to create namespace for env: ${NAMESPACE} - stopping"
            exit 1
        fi
    fi
    deploy_rust_rest_api_secrets_jwt
    deploy_rust_rest_api_secrets_tls
    deploy_rust_rest_db_secrets_tls
    deploy_rust_rest_pgadmin_secrets_tls
    deploy_postgres_db_credentials
    deploy_api_db_credentials
    deploy_s3_write_credentials
} # deploy_tls_secrets - end

deploy_tls_secrets

green "done deploying tls assets"

exit 0
