#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

export TLS_ARGS=""
if [[ "${API_ENDPOINT}" == "" ]]; then
    red "please set the environment variable: API_ENDPOINT to something like: export API_ENDPOINT=api.yourdomain.com"
    exit 1
fi

function build_auth_header() {
    export API_AUTH_HEADER="Bearer: ${API_TOKEN}"
} # build_auth_header - end

function user_create() {
    username="user@email.com"
    password="123321"
    if [[ "${1}" != "" ]]; then
        username="${1}"
    fi
    if [[ "${2}" != "" ]]; then
        password="${2}"
    fi
    echo "creating user: ${username}"
    create_user_out=$(curl -s "${TLS_ARGS}" \
        "https://${API_ENDPOINT}/user" \
        -XPOST \
        -d "{\"email\":\"${username}\",\"password\":\"${password}\"}")
    last_status="$?"
    if [[ "${last_status}" -ne 0 ]]; then
        red "failed to create user: ${username} - stopping"
        exit 1
    fi
    api_token=$(echo "${create_user_out}" | jq -r '.token')
    last_status="$?"
    if [[ "${last_status}" -ne 0 ]]; then
        red "failed to find new user: ${username} token - stopping"
        echo -e "\n${create_user_out}\n"
        exit 1
    fi
    user_id=$(echo "${create_user_out}" | jq -r '.user_id')
    export API_TOKEN="${api_token}"
    build_auth_header
    export API_USERNAME="${username}"
    export API_PASSWORD="${password}"
    export API_USER_ID="${user_id}"
    if [[ "${API_USER_ID}" == "" ]] || [[ "${API_USER_ID}" == "null" ]]; then
        red "failed to get token on new user ${username} - stopping"
        echo -e "\nuser out: ${create_user_out}\n"
        exit 1
    fi
} # user_create - end

function user_login() {
    username="user@email.com"
    password="123321"
    if [[ "${1}" != "" ]]; then
        username="${1}"
    fi
    if [[ "${2}" != "" ]]; then
        password="${2}"
    fi
    echo "logging user in: ${username}"
    login_user_out=$(curl -s "${TLS_ARGS}" \
        "https://${API_ENDPOINT}/login" \
        -XPOST \
        -d "{\"email\":\"${username}\",\"password\":\"${password}\"}")
    last_status="$?"
    if [[ "${last_status}" -ne 0 ]]; then
        echo "failed to login user: ${username} - stopping"
        exit 1
    fi
    api_token=$(echo "${login_user_out}" | jq -r '.token')
    last_status="$?"
    if [[ "${last_status}" -ne 0 ]]; then
        echo "failed to find user to login: ${username} token - stopping"
        echo -e "\n${login_user_out}\n"
        exit 1
    fi
    user_id=$(echo "${login_user_out}" | jq -r '.user_id')
    export API_TOKEN="${api_token}"
    build_auth_header
    export API_USERNAME="${username}"
    export API_PASSWORD="${password}"
    export API_USER_ID="${user_id}"
    if [[ "${API_USER_ID}" == "" ]] || [[ "${API_USER_ID}" == "null" ]]; then
        red "failed to get token on user loging ${username} - stopping"
        echo -e "\nuser out: ${login_user_out}\n"
        exit 1
    fi
} # user_login - end

function user_get() {
    if [[ "${API_TOKEN}" == "" ]]; then
        user_login "${API_USERNAME}" "${API_PASSWORD}"
    fi
    user_id="${API_USER_ID}"
    echo "getting user id: ${user_id}"
    get_user_out=$(
        curl -s "${TLS_ARGS}" \
            "https://${API_ENDPOINT}/user/${user_id}" \
            -XGET \
            -H 'Content-Type: application/json' \
            -H 'Accept: application/json' \
            -H "${API_AUTH_HEADER}"
    )
    last_status="$?"
    if [[ "${last_status}" -ne 0 ]]; then
        red "failed to get user: ${user_id} - stopping"
        echo -e "\n${get_user_out}\nurl: https://${API_ENDPOINT}/user/${user_id}\ntoken header: ${API_AUTH_HEADER}"
        exit 1
    fi
    api_token=$(echo "${get_user_out}" | jq -r '.user_id')
    last_status="$?"
    if [[ "${last_status}" -ne 0 ]]; then
        echo "failed to get user_id in output: ${API_USERNAME} - stopping"
        echo -e "\n${get_user_out}\n"
        exit 1
    fi
    export API_TOKEN="${api_token}"
} # user_get - end

function run_tests() {
    max_tests=20
    yellow "running ${max_tests} tests on ${API_ENDPOINT}"

    for (( c=0; c<="$max_tests"; c++ ))
    do
        new_user_suffix=$(date -u +'%Y%m%d%H%M%S%N')
        test_username="testuser${new_user_suffix}@email.com"
        test_user_password="testuser${user_id}"
        yellow "running test ${c}"
        user_create "${test_username}" "${test_user_password}"
        user_login "${test_username}" "${test_user_password}"
        user_get
        user_get
        user_get
        user_get
        user_get
        user_get
        user_login "${test_username}" "${test_user_password}"
    done
} # run_tests - end

run_tests

exit 0
