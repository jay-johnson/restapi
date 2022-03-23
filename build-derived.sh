#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

function build_rust_restapi_artifact() {
    yellow "building rust rest api derived image"
    time docker build -f ./derived.Dockerfile --rm -t jayjohnson/rust-restapi:latest .
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        red "error - failed to build rust rest api derived image - stopping"
        exit 1
    fi
} # build_rust_restapi_artifact - end

build_rust_restapi_artifact

green "done building docker derived image"

exit 0
