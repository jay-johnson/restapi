#!/bin/bash

function recreate_jwt() {
    echo "creating JWT private and public signing keys"
    openssl ecparam -name prime256v1 -genkey -out private-key.pem
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        echo "failed to create private-key.pem - stopping"
    fi
    openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out private-key-pkcs8.pem
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        echo "failed to create pkcs8 from private-key.pem - stopping"
    fi
    openssl ec -in private-key.pem -pubout -out public-key.pem
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        echo "failed to create public-key.pem - stopping"
    fi

    echo "done creating JWT private and public signing keys"
}

recreate_jwt

exit 0
