#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

which cfssl > /dev/null 2>&1
lt="$?"
if [[ "${lt}" -ne 0 ]]; then
    red "please install cfssl and retry"
    exit 1
fi
which keytool > /dev/null 2>&1
lt="$?"
if [[ "${lt}" -ne 0 ]]; then
    red "please install keytool and retry"
    exit 1
fi
which openssl > /dev/null 2>&1
lt="$?"
if [[ "${lt}" -ne 0 ]]; then
    red "please install openssl and retry"
    exit 1
fi
which uuidgen > /dev/null 2>&1
lt="$?"
if [[ "${lt}" -ne 0 ]]; then
    red "please install uuidgen and retry"
    exit 1
fi

PATH_TO_CA_CONFIG="../ca/cfssl-ca.json"
PATH_TO_CA_FILE="../ca/ca.pem"
PATH_TO_CA_KEY="../ca/ca-key.pem"

# note: each time you run this tool, it will roll the passwords if you do not set them
# ahead of time
if [[ "${PASSWORD_SERVER_KEYSTORE}" == "" ]]; then
    new_pw=$(uuidgen | sed -e 's/-//g')
    export PASSWORD_SERVER_KEYSTORE="${new_pw}"
fi
if [[ "${PASSWORD_CLIENT_KEYSTORE}" == "" ]]; then
    new_pw=$(uuidgen | sed -e 's/-//g')
    export PASSWORD_CLIENT_KEYSTORE="${new_pw}"
fi
if [[ "${PASSWORD_PEER_KEYSTORE}" == "" ]]; then
    new_pw=$(uuidgen | sed -e 's/-//g')
    export PASSWORD_PEER_KEYSTORE="${new_pw}"
fi

function build_ca() {
    if [[ ! -e ./ca/ca-key.pem ]] && [[ ! -e ./ca/ca.pem ]] && [[ ! -e ./ca/ca.csr ]]; then
        cd ./ca || return
        rm ./*.p12 ./*.txt ./*.pem ./*.csr ./*.pkcs12 ./*.jks > /dev/null 2>&1
        yellow "generating new certificate authority ca_file=${PATH_TO_CA_FILE} key=${PATH_TO_CA_KEY}"
        cfssl gencert -initca cfssl-ca.json | cfssljson -bare ca
        lt="$?"
        if [[ "${lt}" -ne 0 ]]; then
            red "${tls_name} - failed to generate ca with: cfssl gencert -initca cfssl-ca.json | cfssljson -bare ca"
            exit 1
        fi
        cd ..
    else
        green "not recreating CA pem and key - if you want to recreate the ca: rm -f ./ca/ca-key.pem ./ca/ca.pem ./ca/ca.csr"
    fi
}

function build_tls_asset() {
    tls_name="${1}"
    echo "removing previous ${tls_name} assets"
    rm ./*.p12 ./*.txt ./*.pem ./*.csr ./*.pkcs12 ./*.jks > /dev/null 2>&1
    path_to_tls_config="./cfssl-server-csr.json"
    echo "generating ${tls_name} - server"
    cfssl gencert -ca "${PATH_TO_CA_FILE}" -ca-key "${PATH_TO_CA_KEY}" -config "${PATH_TO_CA_CONFIG}" -profile server "${path_to_tls_config}" | cfssljson -bare server
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to generate server with: cfssl gencert -ca ${PATH_TO_CA_FILE} -ca-key ${PATH_TO_CA_KEY} -config ${PATH_TO_CA_CONFIG} -profile server ${path_to_tls_config} | cfssljson -bare server"
        exit 1
    fi
    echo "generating ${tls_name} - peer"
    cfssl gencert -ca "${PATH_TO_CA_FILE}" -ca-key "${PATH_TO_CA_KEY}" -config "${PATH_TO_CA_CONFIG}" -profile server "${path_to_tls_config}" | cfssljson -bare peer
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to generate peer with: cfssl gencert -ca ${PATH_TO_CA_FILE} -ca-key ${PATH_TO_CA_KEY} -config ${PATH_TO_CA_CONFIG} -profile server ${path_to_tls_config} | cfssljson -bare peer"
        exit 1
    fi
    echo "generating ${tls_name} - client"
    cfssl gencert -ca "${PATH_TO_CA_FILE}" -ca-key "${PATH_TO_CA_KEY}" -config "${PATH_TO_CA_CONFIG}" -profile client "${path_to_tls_config}" | cfssljson -bare client
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to generate client with: cfssl gencert -ca ${PATH_TO_CA_FILE} -ca-key ${PATH_TO_CA_KEY} -config ${PATH_TO_CA_CONFIG} -profile client ${path_to_tls_config} | cfssljson -bare client"
        exit 1
    fi
    echo "generating ${tls_name} - server pkcs12 keystore"
    # no intermediate certs assume this is coming soon
    cat ./server.pem "${PATH_TO_CA_FILE}" > ./server-cert-chain.pem
    vc="openssl pkcs12 -export -certfile ${PATH_TO_CA_FILE} -in ./server-cert-chain.pem -inkey ./server-key.pem -name ${tls_name} -out server-keystore.p12 -passout pass:${PASSWORD_SERVER_KEYSTORE} -passin pass:${PASSWORD_SERVER_KEYSTORE} "
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create server pkcs12 keystore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "verifying ${tls_name} - server pkcs12 keystore"
    vc="keytool -list -storetype PKCS12 -keystore ./server-keystore.p12 -storepass ${PASSWORD_SERVER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to verify server pkcs12 keystore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "generating ${tls_name} - server pkcs12 truststore"
    vc="keytool -importcert -storetype PKCS12 -keystore server-truststore.p12 -storepass ${PASSWORD_SERVER_KEYSTORE} -alias CARoot -file ./server-cert-chain.pem -noprompt"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to generate server pkcs12 truststore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "verifiying ${tls_name} - server pkcs12 truststore"
    vc="keytool -list -storetype PKCS12 -keystore server-truststore.p12 -storepass ${PASSWORD_SERVER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to verify server pkcs12 truststore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "creating ${tls_name} - server password file"
    echo "${PASSWORD_SERVER_KEYSTORE}" > ./server-keystore-password.txt
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create server keystore password file with command: ${vc} - stopping"
        exit 1
    fi

    echo "generating ${tls_name} - client pkcs12 keystore"
    # no intermediate certs assume this is coming soon
    cat ./client.pem "${PATH_TO_CA_FILE}" > ./client-cert-chain.pem
    vc="openssl pkcs12 -export -certfile ${PATH_TO_CA_FILE} -in ./client-cert-chain.pem -inkey ./client-key.pem -name ${tls_name} -out client-keystore.p12 -passout pass:${PASSWORD_CLIENT_KEYSTORE} -passin pass:${PASSWORD_CLIENT_KEYSTORE} "
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create client pkcs12 file with command: ${vc} - stopping"
        exit 1
    fi
    echo "verifying ${tls_name} - client pkcs12 keystore"
    vc="keytool -list -storetype PKCS12 -keystore ./client-keystore.p12 -storepass ${PASSWORD_CLIENT_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to verify client pkcs12 keystore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "generating ${tls_name} - client pkcs12 truststore"
    vc="keytool -importcert -storetype PKCS12 -keystore client-truststore.p12 -storepass ${PASSWORD_CLIENT_KEYSTORE} -alias CARoot -file ./client-cert-chain.pem -noprompt"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to generate client pkcs12 truststore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "verifiying ${tls_name} - client pkcs12 truststore"
    vc="keytool -list -storetype PKCS12 -keystore client-truststore.p12 -storepass ${PASSWORD_CLIENT_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to verify client pkcs12 truststore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "creating ${tls_name} - client password file"
    echo "${PASSWORD_CLIENT_KEYSTORE}" > ./client-keystore-password.txt
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create client keystore password file with command: ${vc} - stopping"
        exit 1
    fi

    echo "generating ${tls_name} - peer pkcs12 keystore"
    # no intermediate certs assume this is coming soon
    cat ./peer.pem "${PATH_TO_CA_FILE}" > ./peer-cert-chain.pem
    vc="openssl pkcs12 -export -certfile ${PATH_TO_CA_FILE} -in ./peer-cert-chain.pem -inkey ./peer-key.pem -name ${tls_name} -out peer-keystore.p12 -passout pass:${PASSWORD_PEER_KEYSTORE} -passin pass:${PASSWORD_PEER_KEYSTORE} "
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create peer pkcs12 file with command: ${vc} - stopping"
        exit 1
    fi
    echo "verifying ${tls_name} - peer pkcs12 keystore"
    vc="keytool -list -storetype PKCS12 -keystore ./peer-keystore.p12 -storepass ${PASSWORD_PEER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to verify peer pkcs12 keystore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "generating ${tls_name} - peer pkcs12 truststore"
    vc="keytool -importcert -storetype PKCS12 -keystore peer-truststore.p12 -storepass ${PASSWORD_PEER_KEYSTORE} -alias CARoot -file ./peer-cert-chain.pem -noprompt"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to generate peer pkcs12 truststore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "verifiying ${tls_name} - peer pkcs12 truststore"
    vc="keytool -list -storetype PKCS12 -keystore peer-truststore.p12 -storepass ${PASSWORD_PEER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to verify peer pkcs12 truststore file with command: ${vc} - stopping"
        exit 1
    fi
    echo "creating ${tls_name} - peer password file"
    echo "${PASSWORD_PEER_KEYSTORE}" > ./peer-keystore-password.txt
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create peer keystore password file with command: ${vc} - stopping"
        exit 1
    fi

    # server jks
    yellow "converting ${tls_name} - pkcs12 server-keystore.p12 to jks server-keystore.jks"
    vc="keytool -importkeystore -deststorepass ${PASSWORD_SERVER_KEYSTORE} -destkeypass ${PASSWORD_SERVER_KEYSTORE} -destkeystore server-keystore.jks -srckeystore server-keystore.p12 -srcstoretype PKCS12 -srcstorepass ${PASSWORD_SERVER_KEYSTORE} -alias ${tls_name}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to convert pkcs12 keystore to jks keystore command: ${vc} - stopping"
        exit 1
    fi
    echo "${PASSWORD_SERVER_KEYSTORE}" > ./server-jks-keystore-password.txt
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create server jks keystore password file with command: ${vc} - stopping"
        exit 1
    fi
    yellow "import ${tls_name} - keystore jks with alias=CARoot and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore server-keystore.jks -noprompt -alias CARoot -import -file ${PATH_TO_CA_FILE} -storepass ${PASSWORD_SERVER_KEYSTORE} -keypass ${PASSWORD_SERVER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to import CA=${PATH_TO_CA_FILE} with command: ${vc} - stopping"
        echo -e "\nkeytool -exportcert -alias CARoot -keystore ${PATH_TO_CA_FILE} -storepass ${PASSWORD_SERVER_KEYSTORE}\n"
        exit 1
    fi
    yellow "import ${tls_name} - keystore jks chain alias=${tls_name} and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore server-keystore.jks -noprompt -alias ${tls_name} -import -file server-cert-chain.pem -storepass ${PASSWORD_SERVER_KEYSTORE} -keypass ${PASSWORD_SERVER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to import server cert chain into jks file with command: ${vc} - stopping"
        echo -e "\nkeytool -exportcert -alias ${tls_name} -keystore ./server-keystore.jks -storepass ${PASSWORD_SERVER_KEYSTORE}\n" 
        exit 1
    fi
    yellow "creating ${tls_name} - truststore jks with alias=CARoot and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore server-truststore.jks -noprompt -alias CARoot -import -file ${PATH_TO_CA_FILE} -storepass ${PASSWORD_SERVER_KEYSTORE} -keypass ${PASSWORD_SERVER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create server truststore jks file with command: ${vc} - stopping"
        exit 1
    fi

    # client jks
    yellow "converting ${tls_name} - pkcs12 client-keystore.p12 to jks client-keystore.jks"
    vc="keytool -importkeystore -deststorepass ${PASSWORD_CLIENT_KEYSTORE} -destkeypass ${PASSWORD_CLIENT_KEYSTORE} -destkeystore client-keystore.jks -srckeystore client-keystore.p12 -srcstoretype PKCS12 -srcstorepass ${PASSWORD_CLIENT_KEYSTORE} -alias ${tls_name}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to convert pkcs12 keystore to jks keystore command: ${vc} - stopping"
        exit 1
    fi
    echo "${PASSWORD_CLIENT_KEYSTORE}" > ./client-jks-keystore-password.txt
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create client jks keystore password file with command: ${vc} - stopping"
        exit 1
    fi
    yellow "import ${tls_name} - keystore jks with alias=CARoot and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore client-keystore.jks -noprompt -alias CARoot -import -file ${PATH_TO_CA_FILE} -storepass ${PASSWORD_CLIENT_KEYSTORE} -keypass ${PASSWORD_CLIENT_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to import CA=${PATH_TO_CA_FILE} with command: ${vc} - stopping"
        echo -e "\nkeytool -exportcert -alias CARoot -keystore ${PATH_TO_CA_FILE} -storepass ${PASSWORD_CLIENT_KEYSTORE}\n"
        exit 1
    fi
    yellow "import ${tls_name} - keystore jks chain alias=${tls_name} and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore client-keystore.jks -noprompt -alias ${tls_name} -import -file client-cert-chain.pem -storepass ${PASSWORD_CLIENT_KEYSTORE} -keypass ${PASSWORD_CLIENT_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to import client cert chain into jks file with command: ${vc} - stopping"
        echo -e "\nkeytool -exportcert -alias ${tls_name} -keystore ./client-keystore.jks -storepass ${PASSWORD_CLIENT_KEYSTORE}\n" 
        exit 1
    fi
    yellow "creating ${tls_name} - truststore jks with alias=CARoot and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore client-truststore.jks -noprompt -alias CARoot -import -file ${PATH_TO_CA_FILE} -storepass ${PASSWORD_CLIENT_KEYSTORE} -keypass ${PASSWORD_CLIENT_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create client truststore jks file with command: ${vc} - stopping"
        exit 1
    fi

    # peer jks
    yellow "converting ${tls_name} - pkcs12 peer-keystore.p12 to jks peer-keystore.jks"
    vc="keytool -importkeystore -deststorepass ${PASSWORD_PEER_KEYSTORE} -destkeypass ${PASSWORD_PEER_KEYSTORE} -destkeystore peer-keystore.jks -srckeystore peer-keystore.p12 -srcstoretype PKCS12 -srcstorepass ${PASSWORD_PEER_KEYSTORE} -alias ${tls_name}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to convert pkcs12 keystore to jks keystore command: ${vc} - stopping"
        exit 1
    fi
    echo "${PASSWORD_PEER_KEYSTORE}" > ./peer-jks-keystore-password.txt
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create peer jks keystore password file with command: ${vc} - stopping"
        exit 1
    fi
    yellow "import ${tls_name} - keystore jks with alias=CARoot and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore peer-keystore.jks -noprompt -alias CARoot -import -file ${PATH_TO_CA_FILE} -storepass ${PASSWORD_PEER_KEYSTORE} -keypass ${PASSWORD_PEER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to import CA=${PATH_TO_CA_FILE} with command: ${vc} - stopping"
        echo -e "\nkeytool -exportcert -alias CARoot -keystore ${PATH_TO_CA_FILE} -storepass ${PASSWORD_PEER_KEYSTORE}\n"
        exit 1
    fi
    yellow "import ${tls_name} - keystore jks chain alias=${tls_name} and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore peer-keystore.jks -noprompt -alias ${tls_name} -import -file peer-cert-chain.pem -storepass ${PASSWORD_PEER_KEYSTORE} -keypass ${PASSWORD_PEER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to import peer cert chain into jks file with command: ${vc} - stopping"
        echo -e "\nkeytool -exportcert -alias ${tls_name} -keystore ./peer-keystore.jks -storepass ${PASSWORD_PEER_KEYSTORE}\n" 
        exit 1
    fi
    yellow "creating ${tls_name} - truststore jks with alias=CARoot and CA=${PATH_TO_CA_FILE}"
    vc="keytool -keystore peer-truststore.jks -noprompt -alias CARoot -import -file ${PATH_TO_CA_FILE} -storepass ${PASSWORD_PEER_KEYSTORE} -keypass ${PASSWORD_PEER_KEYSTORE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "${tls_name} - failed to create peer truststore jks file with command: ${vc} - stopping"
        exit 1
    fi
}

function build_all_assets() {
    tls_assets=$(ls ./*/cfssl-server-csr.json)
    for tls_asset in ${tls_assets}; do
        asset_name=$(dirname "${tls_asset}" | sed -e 's|\.\/||g')
        yellow "building: ${asset_name}"
        cd "${asset_name}" || red "failed to change to asset dir: ${asset_name}"
        build_tls_asset "${asset_name}"
        cd ..
    done
}

build_ca
build_all_assets

green "done"

exit 0
