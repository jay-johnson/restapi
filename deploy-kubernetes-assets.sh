#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

yellow "deploying assets into kubernetes"

if [[ "${ENV_NAME}" == "" ]]; then
    ENV_NAME="default"
fi
if [[ "${DB_API_USERNAME}" == "" ]]; then
    export DB_API_USERNAME="datawriter"
fi
if [[ "${DB_API_PASSWORD}" == "" ]]; then
    export DB_API_PASSWORD="123321"
fi
if [[ "${USE_AWS_S3_AK}" == "" ]]; then
    export USE_AWS_S3_AK="ACCESS_KEY_HERE"
fi
if [[ "${USE_AWS_S3_SK}" == "" ]]; then
    export USE_AWS_S3_SK="SECRET_KEY_HERE"
fi

function usage() {
    echo ""
    yellow "--------------------------"
    yellow "usage:"
    echo ""
    yellow "${0} has supported arguments:"
    echo ""
    echo " -a - AWS S3 access key for the restapi - (default: ${USE_AWS_S3_AK})"
    echo " -e - ENV NAMESPACE - e.g. <default|dev|qa|test|uat|prod> (default: ${ENV_NAME})"
    echo " -u - api postgres database username - (default: ${DB_API_USERNAME})"
    echo " -p - api postgres database password - (default: ${DB_API_PASSWORD})"
    echo " -s - AWS S3 secret key for the restapi - (default: ${USE_AWS_S3_SK})"
    echo ""
}

while getopts ":a:e:p:s:u:h" o; do
    case "${o}" in
    a)
        export USE_AWS_S3_AK="${OPTARG}"
        ;;
    e)
        export ENV_NAME="${OPTARG}"
        ;;
    p)
        export DB_API_PASSWORD="${OPTARG}"
        ;;
    s)
        export USE_AWS_S3_SK="${OPTARG}"
        ;;
    u)
        export DB_API_USERNAME="${OPTARG}"
        ;;
    h)
        usage
        exit 0
        ;;
    *)
        usage
        echo ""
        err "unsupported argument: ${o} - stopping"
        echo ""
        exit 1
        ;;
    esac
done

function deploy_rust_rest_api_secrets_jwt() {
    var_path_to_jwt_key_private="./jwt/private-key-pkcs8.pem"
    var_path_to_jwt_key_public="./jwt/public-key.pem"
    kubectl delete -n "${ENV_NAME}" secret "${ENV_NAME}-api-jwt-keys" --ignore-not-found
    vc="kubectl create -n ${ENV_NAME} secret generic ${ENV_NAME}-api-jwt-keys --from-file=private-key.pem=${var_path_to_jwt_key_private} --from-file=public-key.pem=${var_path_to_jwt_key_public}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "error - failed to deploy rust rest api jwt keys as a secret - stopping"
        echo -e "\n${vc}\n"
        exit 1
    fi
} # deploy_rust_rest_api_secrets_jwt - end

function deploy_api_db_credentials() {
    var_secret_name="${ENV_NAME}-api-db-credentials"
    kubectl delete -n "${ENV_NAME}" secret "${var_secret_name}" --ignore-not-found
    kubectl create secret -n "${ENV_NAME}" generic "${var_secret_name}" --from-literal=username="${DB_API_USERNAME}" --from-literal=password="${DB_API_PASSWORD}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to create api db credentials secret: ${var_secret_name} for env: ${ENV_NAME} - stopping"
        exit 1
    fi
} # deploy_api_db_credentials - end

function deploy_s3_credentials() {
    var_secret_name="${ENV_NAME}-api-s3-credentials"
    kubectl delete -n "${ENV_NAME}" secret "${var_secret_name}" --ignore-not-found
    kubectl create secret -n "${ENV_NAME}" generic "${var_secret_name}" --from-literal=access-key="${USE_AWS_S3_AK}" --from-literal=secret-key="${USE_AWS_S3_SK}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to create api s3 credential secret: ${var_secret_name} for env: ${ENV_NAME} - stopping"
        exit 1
    fi
} # deploy_s3_credentials - end

function deploy_cfssl_assets() {
    echo "deploying cfssl tls assets"
    cd ./tls || return
    ./deploy-tls-assets.sh -e "${ENV_NAME}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "error - failed to deploy cfssl tls assets - stopping"
        exit 1
    fi
    cd .. || return
} # deploy_cfssl_assets - end

function deploy_all() {
    echo "creating namespace=${ENV_NAME} if it does not exist"
    kubectl create ns "${ENV_NAME}" > /dev/null 2>&1
    yellow "deploying jwt tls assets as a kubernetes secrets into namespace=${ENV_NAME}"
    deploy_rust_rest_api_secrets_jwt
    yellow "deploying cfssl tls assets as kubernetes secrets into namespace=${ENV_NAME}"
    deploy_cfssl_assets
    yellow "deploying db credentials into namespace=${ENV_NAME}"
    deploy_api_db_credentials
    yellow "deploying s3 credentials into namespace=${ENV_NAME}"
    deploy_s3_credentials
} # deploy_all - end

deploy_all

green "done deploying tls assets"

exit 0
