#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

yellow "deploying tls assets into kubernetes"

function deploy_rust_rest_api_secrets_jwt() {
    var_path_to_jwt_key_private="./jwt/private-key-pkcs8.pem"
    var_path_to_jwt_key_public="./jwt/public-key.pem"
    kubectl delete -n default secret rust-restapi-jwt-keys --ignore-not-found
    var_command="kubectl create -n default secret generic rust-restapi-jwt-keys --from-file=private-key.pem=${var_path_to_jwt_key_private} --from-file=public-key.pem=${var_path_to_jwt_key_public}"
    eval "${var_command}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api jwt keys as a secret - stopping"
        echo -e "\n${var_command}\n"
        exit 1
    fi
} # deploy_rust_rest_api_secrets_jwt - end

function deploy_rust_rest_api_secrets_tls() {
    var_path_to_api_tls_ca="./certs/tls/api/api-ca.pem"
    var_path_to_api_tls_crt="./certs/tls/api/api.crt"
    var_path_to_api_tls_key="./certs/tls/api/api.key"
    kubectl delete -n default secret rust-restapi-tls-api --ignore-not-found
    var_command="kubectl create -n default secret -n default generic rust-restapi-tls-api --from-file=api.crt=${var_path_to_api_tls_crt} --from-file=api.key=${var_path_to_api_tls_key} --from-file=api-ca.pem=${var_path_to_api_tls_ca}"
    eval "${var_command}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api server tls secret - stopping"
        echo -e "\n${var_command}\n"
        exit 1
    fi
} # deploy_rust_rest_api_secrets_tls - end

function deploy_rust_rest_db_secrets_tls() {
    var_path_to_db_tls_ca="./certs/tls/postgres/postgres-ca.pem"
    var_path_to_db_tls_crt="./certs/tls/postgres/postgres.crt"
    var_path_to_db_tls_key="./certs/tls/postgres/postgres.key"
    kubectl delete -n default secret rust-restapi-tls-db --ignore-not-found
    var_command="kubectl create -n default secret generic rust-restapi-tls-db --from-file=postgres.crt=${var_path_to_db_tls_crt} --from-file=postgres.key=${var_path_to_db_tls_key} --from-file=postgres-ca.pem=${var_path_to_db_tls_ca}"
    eval "${var_command}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api database tls secret - stopping"
        echo -e "\n${var_command}\n"
        exit 1
    fi
} # deploy_rust_rest_api_secrets_tls - end

function deploy_rust_rest_pgadmin_secrets_tls() {
    var_path_to_pgadmin_tls_ca="./certs/tls/pgadmin/pgadmin-ca.pem"
    var_path_to_pgadmin_tls_crt="./certs/tls/pgadmin/pgadmin.crt"
    var_path_to_pgadmin_tls_key="./certs/tls/pgadmin/pgadmin.key"
    kubectl delete -n default secret rust-restapi-tls-pgadmin --ignore-not-found
    # https://www.pgadmin.org/docs/pgadmin4/6.7/container_deployment.html#mapped-files-and-directories
    var_command="kubectl create -n default secret generic rust-restapi-tls-pgadmin --from-file=server.cert=${var_path_to_pgadmin_tls_crt} --from-file=server.key=${var_path_to_pgadmin_tls_key} --from-file=pgadmin-ca.pem=${var_path_to_pgadmin_tls_ca}"
    eval "${var_command}"
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api pgadmin tls secret - stopping"
        echo -e "\n${var_command}\n"
        exit 1
    fi
} # deploy_rust_rest_pgadmin_secrets_tls - end

function deploy_tls_secrets() {
    yellow "deploying tls and jwt secrets"
    deploy_rust_rest_api_secrets_jwt
    deploy_rust_rest_api_secrets_tls
    deploy_rust_rest_db_secrets_tls
    deploy_rust_rest_pgadmin_secrets_tls
} # deploy_tls_secrets - end

deploy_tls_secrets

green "done deploying tls assets"

exit 0
