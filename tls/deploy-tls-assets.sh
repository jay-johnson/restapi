#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

which kubectl > /dev/null 2>&1
lt="$?"
if [[ "${lt}" -ne 0 ]]; then
    red "please install kubectl and ensure you have a valid kubernetes cluster running and retry (deploy-tls-assets.sh was tested on minikube: 1.24.3)"
    exit 1
fi

if [[ "${ENV_NAME}" == "" ]]; then
    ENV_NAME="default"
fi
if [[ "${PATH_TO_CA_FILE}" == "" ]]; then
    PATH_TO_CA_FILE="../ca/ca.pem"
fi
if [[ "${PATH_TO_CA_KEY}" == "" ]]; then
    PATH_TO_CA_KEY="../ca/ca-key.pem"
fi

function usage() {
    echo ""
    yellow "--------------------------"
    yellow "usage:"
    echo ""
    yellow "${0} has supported arguments:"
    echo ""
    echo " -e - ENV NAMESPACE - e.g. <default|dev|qa|test|uat|prod> (default: ${ENV_NAME})"
    echo ""
}

while getopts ":e:h" o; do
    case "${o}" in
    e)
        export ENV_NAME="${OPTARG}"
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

function deploy_tls_asset() {
    tls_asset_name="${1}"
    secret_name="${2}"
    secret_namespace="${3}"
    ca_file="${4}"
    ca_key="${5}"
    crt_file="${6}"
    key_file="${7}"
    
    yellow "deploying: ${tls_asset_name} as ${secret_namespace}/${secret_name}"
    vc="kubectl delete secret ${secret_name} -n ${secret_namespace} --ignore-not-found > /dev/null 2>&1"
    eval "${vc}"
    custom_args=""

    if [[ "${tls_asset_name}" == "pgadmin" ]]; then
        custom_args="--from-file=server.cert=server.pem --from-file=server.key=server-key.pem"
    elif [[ "${tls_asset_name}" == "kafka-cluster-0" ]]; then
        if [[ "${secret_name}" == "tls-kafka-cluster-0-server" ]]; then

            echo "deploying strimzi kafka tls assets and building custom arguments"
            # https://banzaicloud.com/docs/supertubes/kafka-operator/ssl/#using-own-certificates
            custom_args="--from-file=keystore.jks=./server-keystore.jks --from-file=truststore.jks=./server-truststore.jks --from-file=password=./server-keystore-password.txt --from-file=ca.crt=${ca_file} --from-file=tls.crt=./server.pem --from-file=tls.key=./server-key.pem --from-file=clientCert=./client.pem --from-file=clientKey=./client-key.pem --from-file=peerCert=./peer.pem --from-file=peerKey=./peer-key.pem --from-file=serverCert=./server.pem --from-file=serverKey=./server-key.pem"
            kubectl delete secret -n "${secret_namespace}" "${secret_namespace}-cluster-ca-cert" --ignore-not-found
            kubectl create secret -n "${secret_namespace}" generic "${secret_namespace}-cluster-ca-cert" --from-file=ca.crt="${ca_file}"
            lt="$?"
            if [[ "${lt}" -ne 0 ]]; then
                red "failed to create ${secret_namespace} kafka cluster-ca secret for ca.crt - stopping"
                exit 1
            fi
            kubectl label secret -n "${secret_namespace}" "${secret_namespace}-cluster-ca-cert" strimzi.io/kind=Kafka strimzi.io/cluster="${secret_namespace}"
            kubectl annotate secret -n "${secret_namespace}" "${secret_namespace}-cluster-ca-cert" strimzi.io/ca-cert-generation=0

            kubectl delete secret -n "${secret_namespace}" "${secret_namespace}-cluster-ca" --ignore-not-found
            kubectl create secret -n "${secret_namespace}" generic "${secret_namespace}-cluster-ca" --from-file=ca.key="${ca_key}"
            lt="$?"
            if [[ "${lt}" -ne 0 ]]; then
                red "failed to create ${secret_namespace} kafka cluster-ca secret for ca.key - stopping"
                exit 1
            fi
            kubectl label secret -n "${secret_namespace}" "${secret_namespace}-cluster-ca" strimzi.io/kind=Kafka strimzi.io/cluster="${secret_namespace}"
            kubectl annotate secret -n "${secret_namespace}" "${secret_namespace}-cluster-ca" strimzi.io/ca-key-generation=0

            # load the client CA pem
            # https://github.com/scholzj/strimzi-custom-ca-test/blob/de7218414501084851e30aff21bf3ff58dad1a68/load.sh#L23
            kubectl delete secret -n "${secret_namespace}" "${secret_namespace}-clients-ca-cert" --ignore-not-found
            kubectl create secret -n "${secret_namespace}" generic "${secret_namespace}-clients-ca-cert" --from-file=ca.crt="${ca_file}"
            lt="$?"
            if [[ "${lt}" -ne 0 ]]; then
                red "failed to create ${secret_namespace} kafka clients-ca-cert secret for ca.crt - stopping"
                exit 1
            fi
            kubectl label secret -n "${secret_namespace}" "${secret_namespace}-clients-ca-cert" strimzi.io/kind=Kafka strimzi.io/cluster="${secret_namespace}"
            kubectl annotate secret -n "${secret_namespace}" "${secret_namespace}-clients-ca-cert" strimzi.io/ca-cert-generation=0

            # load the client CA key
            kubectl delete secret -n "${secret_namespace}" "${secret_namespace}-clients-ca" --ignore-not-found
            kubectl create secret -n "${secret_namespace}" generic "${secret_namespace}-clients-ca" --from-file=ca.key="${ca_key}"
            lt="$?"
            if [[ "${lt}" -ne 0 ]]; then
                red "failed to create ${secret_namespace} kafka clients-ca secret for ca.key - stopping"
                exit 1
            fi
            kubectl label secret -n "${secret_namespace}" "${secret_namespace}-clients-ca" strimzi.io/kind=Kafka strimzi.io/cluster="${secret_namespace}"
            kubectl annotate secret -n "${secret_namespace}" "${secret_namespace}-clients-ca" strimzi.io/ca-key-generation=0
        elif [[ "${secret_name}" == "tls-kafka-cluster-0-client" ]]; then
            custom_args="--from-file=ca.crt=${ca_file} --from-file=tls.crt=./client.pem --from-file=tls.key=./client-key.pem --from-file=clientCert=./client.pem --from-file=clientKey=./client-key.pem --from-file=peerCert=./peer.pem --from-file=peerKey=./peer-key.pem --from-file=kafka-key.pem=./client-key.pem --from-file=kafka-crt.pem=./client.pem --from-file=kafka-ca.pem=${ca_file}"
        fi
    fi

    vc="kubectl create secret generic ${secret_name} -n ${secret_namespace} --from-file=${tls_asset_name}-ca.pem=${ca_file} --from-file=${tls_asset_name}-crt.pem=./${crt_file} --from-file=${tls_asset_name}-key.pem=./${key_file} --from-file=caCert=./${ca_file} --from-file=caKey=${ca_key} --from-file=server-keystore.p12=./server-keystore.p12 --from-file=client-keystore.p12=./client-keystore.p12 --from-file=server-cert-chain.pem=./server-cert-chain.pem --from-file=client-cert-chain.pem=./client-cert-chain.pem --from-file=client-keystore-password=./client-keystore-password.txt --from-file=server-keystore-password=./server-keystore-password.txt ${custom_args}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to deploy tls asset ${secret_name} with command: ${vc}"
        pwd
        echo -e "\n- check the ca with: kubectl get secret -n ${secret_namespace} -o yaml ${secret_name} | grep ${secret_name}-ca.pem  | awk '{print \$NF}' | base64 -d | openssl x509 -text\n- check the crt with: kubectl get secret -n ${secret_namespace} -o yaml ${secret_name} | grep ${secret_name}-crt.pem  | awk '{print \$NF}' | base64 -d | openssl x509 -text\n- check the key with: kubectl get secret -n ${secret_namespace} -o yaml ${secret_name} | grep ${secret_name}-crt.pem  | awk '{print \$NF}' | base64 -d | openssl rsa -text"
        exit 1
    else
        green " - deployed: ${tls_asset_name} as ${secret_namespace}/${secret_name}"
    fi
}

function deploy_all_assets() {
    yellow "deploying tls assets into: ${ENV_NAME} namespace"
    kubectl create ns "${ENV_NAME}" > /dev/null 2>&1
    tls_assets=$(ls ./*/cfssl-server-csr.json)
    for tls_asset in ${tls_assets}; do
        asset_name=$(dirname "${tls_asset}" | sed -e 's|\.\/||g')
        cd "${asset_name}" || red "failed to change to asset dir: ${asset_name}"
        case "${asset_name}" in
        "api")
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        "control-center")
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        "kafka-cluster-0")
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        "pgadmin")
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        "postgres")
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        "rest-proxy")
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        "schema-registry")
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        "zookeeper")
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "${ENV_NAME}" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        *)
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-server" "default" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "server.pem" "server-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-peer" "default" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "peer.pem" "peer-key.pem"
            deploy_tls_asset "${asset_name}" "tls-${asset_name}-client" "default" "${PATH_TO_CA_FILE}" "${PATH_TO_CA_KEY}" "client.pem" "client-key.pem"
            ;;
        esac
        cd ..
    done
}

deploy_all_assets

green "done"

exit 0
