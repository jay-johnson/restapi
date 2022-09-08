#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

if [[ "${IMAGE_NAME}" == "" ]]; then
    IMAGE_NAME="docker.io/jayjohnson/rust-restapi"
fi

function build_rust_restapi_artifact() {
    yellow "building rust rest api derived image"
    cur_tag=$(grep version Cargo.toml | head -1 | sed -e 's/"//g' | awk '{print $NF}')
    image_with_tag="${IMAGE_NAME}:${cur_tag}"
    echo "time podman build -f ./derived.Dockerfile --rm -t \"${image_with_tag}\" ."
    time podman build --no-cache -f ./derived.Dockerfile --rm -t "${image_with_tag}" .
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "error - failed to build rust rest api derived image - stopping"
        exit 1
    fi
    yellow "getting latest podman image id"
    vc="podman images | grep \"${IMAGE_NAME} \" | grep \"${cur_tag} \" | head -1 | awk '{print \$3}'"
    image_id=$(eval "${vc}")
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "error - failed to find rust rest api derived image - stopping"
        echo -e "\n${vc}\n"
        exit 1
    fi
    if [[ "${image_id}" == "" ]]; then
        red "error - failed to find rust rest api derived image to tag - stopping"
        echo -e "\n${vc}\n"
        exit 1
    fi
    podman tag "${image_id}" "${IMAGE_NAME}:latest"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "error - failed to tag rust rest api derived image - stopping"
        echo -e "\npodman tag ${image_id} ${IMAGE_NAME}:latest\n"
        exit 1
    fi
} # build_rust_restapi_artifact - end

build_rust_restapi_artifact

green "done building docker derived image"

exit 0
