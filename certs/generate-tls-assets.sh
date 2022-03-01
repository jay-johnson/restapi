#!/bin/bash
# shellcheck disable=SC2174

# will create certs in a newly-created directory (by default: ls -lrth ./tls/)
# ./generate-tls-assets.sh -f -c ./configs/dev-network.yml
#

if [[ "${DATE_FMT}" == "" ]]; then
    export DATE_FMT="%Y-%m-%dT%H:%M:%SZ"
fi

if [[ "${VERBOSE}" == "" ]]; then
    export VERBOSE="0"
fi

function determine_directory_path() {
    # find the directory for this file
    var_dir_path_relative=$(dirname "${0}")
    var_dir_path=$(python -c "import os; import sys; print(os.path.realpath(sys.argv[1]))" "${var_dir_path_relative}")
    # set a base path to the hosting directory for this file
    export SDK_BASE_PATH="${var_dir_path}"
    if [[ "${USE_BASE_PATH}" != "" ]]; then
        export SDK_BASE_PATH="${USE_BASE_PATH}"
    fi
} # determine_directory_path - end

determine_directory_path

export DRY_RUN="1"
export USE_LABEL_NAME=""
export USE_TEE_SHIRT=""
export USE_AWS_ACCOUNT_ID=""
export USE_CURRENT_LABEL="generate-new-certs"
export GENERATE_NEW_CERTS="0"
export USE_CERT_FILE=""
export USE_CN="tls-for-demo-stack"
export USE_EXPIRATION_IN_DAYS="365"
export USE_CERT_ZIP_DIR="${SDK_BASE_PATH}/tls"
export USE_CERT_ZIP_LOC="${USE_CERT_ZIP_DIR}"

function light_blue() { printf "\x1b[38;5;045m%s\e[0m " "${@}"; printf "\n"; }
function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

function debug() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ${USE_CURRENT_LABEL} - DEBUG ${*}"
    if [[ ${VERBOSE} -ne 0 ]]; then
        echo "${log_str}"
    fi
} # debug - end

function info() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ${USE_CURRENT_LABEL} - INFO ${*}"
    echo "${log_str}"
} # info - end

function err() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ${USE_CURRENT_LABEL} - ERROR ${*}"
    red "${log_str}"
} # err - end

function trace() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ${USE_CURRENT_LABEL} - TRACE ${*}"
    echo "${log_str}"
} # trace - end

function crit() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ${USE_CURRENT_LABEL} - CRITICAL ${*}"
    warn "${log_str}"
} # crit - end

function good() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ${USE_CURRENT_LABEL} - SUCCESS ${*}"
    if [[ ${SILENT} -eq 0 ]]; then
        green "${log_str}"
    fi
} # good - end

function banner_log() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ${USE_CURRENT_LABEL} - BANNER ${*}"
    yellow "${log_str}"
} # banner_log - end

function header_log() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ${USE_CURRENT_LABEL} - HEADER ${*}"
    light_blue "${log_str}"
} # header_log - end

function usage() {
    echo ""
    yellow "--------------------------"
    yellow "usage:"
    echo ""
    yellow "${0} has supported arguments:"
    echo ""
    echo " -c - string - path to cert file and current value is: ${USE_CERT_FILE}"
    echo " -f - flag - generate new certs even if there are existing ones in the USE_CERT_ZIP_DIR=${USE_CERT_ZIP_DIR} directory and current value is ${GENERATE_NEW_CERTS}"
    echo " -z - string - path to cert creation dir and current value is: ${USE_CERT_ZIP_DIR}"
    echo " -h - flag - show this same usage help message"
    echo ""
} # usage - end

# Call getopt to validate the provided input.
while getopts ":c:fz:h" o; do
    case "${o}" in
    c)
        export USE_CERT_FILE="${OPTARG}"
        ;;
    f)
        export GENERATE_NEW_CERTS="1"
        ;;
    z)
        export USE_CERT_ZIP_DIR="${OPTARG}"
        export USE_CERT_ZIP_LOC="${USE_CERT_ZIP_DIR}"
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

if [[ "${USE_CERT_FILE}" == "" ]]; then
    usage
    err "missing -c PATH_TO_CERT_FILE"
    exit 1
elif [[ ! -e "${USE_CERT_FILE}" ]]; then
    usage
    err "there is no cert file at path: -c PATH_TO_CERT_FILE"
    exit 1
fi

function extract_tls_assets_from_p12s() {
    var_cur_pwd=$(pwd)
    info "extracting tls assets from all dirs with p12's with pwd: ${var_cur_pwd}"
    var_all_dirs=$(ls)

    for var_cert_dir in ${var_all_dirs}; do
        var_extract_p12="./${var_cert_dir}/${var_cert_dir}.p12"
        if [[ -e "./${var_cert_dir}/${var_cert_dir}.p12" ]]; then
            var_key_path="./${var_cert_dir}/${var_cert_dir}.key"
            var_cert_path="./${var_cert_dir}/${var_cert_dir}.crt"
            var_ca_path="./${var_cert_dir}/${var_cert_dir}-ca.pem"
            var_chain_path="./${var_cert_dir}/${var_cert_dir}-chain.pem"

            info "extracting ${var_extract_p12} key=${var_key_path}"
            openssl pkcs12 -in "${var_extract_p12}" -passout pass:'' -passin pass:'' -nocerts -nodes | sed -ne '/-BEGIN PRIVATE KEY-/,/-END PRIVATE KEY-/p' > "${var_key_path}"
            var_last_status="$?"
            if [[ "${var_last_status}" -ne 0 ]]; then
                err "failed to extract KEY: ${var_cert_path} with command:"
                echo ""
                echo "openssl pkcs12 -in \"${var_extract_p12}\" -passout pass:'' -passin pass:'' -nocerts -nodes | sed -ne '/-BEGIN PRIVATE KEY-/,/-END PRIVATE KEY-/p' > \"${var_key_path}\""
                echo ""
                exit 1
            fi

            info "extracting ${var_extract_p12} cert=${var_cert_path}"
            openssl pkcs12 -in "${var_extract_p12}" -clcerts -nokeys -passout pass:'' -passin pass:'' | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > "${var_cert_path}"
            var_last_status="$?"
            if [[ "${var_last_status}" -ne 0 ]]; then
                err "failed to extract CERT: ${var_cert_path} with command:"
                echo ""
                echo "openssl pkcs12 -in \"${var_extract_p12}\" -clcerts -nokeys -passout pass:'' -passin pass:'' | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > \"${var_cert_path}\""
                echo ""
                exit 1
            fi

            info "extracting ${var_extract_p12} ca=${var_ca_path}"
            openssl pkcs12 -in "${var_extract_p12}" -passout pass:'' -passin pass:'' -cacerts -nokeys -chain | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > "${var_ca_path}"
            var_last_status="$?"
            if [[ "${var_last_status}" -ne 0 ]]; then
                err "failed to extract CA: ${var_cert_path} with command:"
                echo ""
                echo "openssl pkcs12 -in \"${var_extract_p12}\" -clcerts -nokeys -passout pass:'' -passin pass:'' | sed -ne '/-BEGIN CERTIFICATE-/,/-END CERTIFICATE-/p' > \"${var_cert_path}\""
                echo ""
                exit 1
            fi

            info "building chain file=${var_ca_path} from cert=${var_cert_path} and ca=${var_ca_path}"
            cp "${var_cert_path}" "${var_chain_path}"
            echo "" >> "${var_chain_path}"
            cat "${var_ca_path}" >> "${var_chain_path}"
        fi
    done
} # extract_tls_assets_from_p12s

function generate_new_certs() {
    var_cur_dir=$(pwd)
    var_last_status="0"
    var_es_keystore_docker_name="elasticsearch-tls-asset-generator"
    export USE_CERT_ZIP_LOC="${USE_CERT_ZIP_DIR}/${var_es_keystore_docker_name}.zip"
    var_use_dirname=$(dirname "${USE_CERT_ZIP_LOC}")
    info "cleaning up any previous cert container"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        docker stop "${var_es_keystore_docker_name}" > /dev/null 2>&1 || true
        docker rm -f "${var_es_keystore_docker_name}" > /dev/null 2>&1 || true
    fi
    if [[ "${var_use_dirname}" != "/" ]] && [[ "${var_use_dirname}" != "//" ]] && [[ "${var_use_dirname}" != "" ]]; then
        trace "cleaning previously-generated certs: ${var_use_dirname}/*"
        rm -rf "${var_use_dirname}"
        # SC2174
        mkdir -m 775 -p "${var_use_dirname}"
    else
        err "invalid USE_CERT_ZIP_LOC=${USE_CERT_ZIP_LOC} - stopping"
        exit 1
    fi
    var_encoded_instances=$(base64 -w 0 "${USE_CERT_FILE}")
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        info "generating certs from encoded ${USE_CERT_FILE}"
        if [[ "${VERBOSE}" -ne 0 ]]; then
            echo ""
            echo "generating certs with command:"
            echo "docker run --name \"${var_es_keystore_docker_name}\" -i -w /app docker.elastic.co/elasticsearch/elasticsearch:7.10.2 /bin/sh -c \"echo ${var_encoded_instances} | base64 -w 0 -d > /app/instances.yaml && elasticsearch-certutil cert --ca-dn CN=${USE_CN} --days ${USE_EXPIRATION_IN_DAYS} --in /app/instances.yaml --out /app/${var_es_keystore_docker_name}.zip --pass ''\""
            echo ""
        fi
        docker run --name "${var_es_keystore_docker_name}" -i -w /app docker.elastic.co/elasticsearch/elasticsearch:7.10.2 /bin/sh -c "echo ${var_encoded_instances} | base64 -w 0 -d > /app/instances.yaml && elasticsearch-certutil cert --ca-dn CN=${USE_CN} --days ${USE_EXPIRATION_IN_DAYS} --in /app/instances.yaml --out /app/${var_es_keystore_docker_name}.zip --pass ''" 2>&1
        var_last_status="$?"
    else
        info " skipped docker run"
    fi
    if [[ "${var_last_status}" -ne 0 ]]; then
        err "failed to generate new elk certs with command:"
        echo ""
        echo "docker run --name \"${var_es_keystore_docker_name}\" -i -w /app docker.elastic.co/elasticsearch/elasticsearch:7.10.2 /bin/sh -c \"echo ${var_encoded_instances} | base64 -w 0 -d > /app/instances.yaml && elasticsearch-certutil cert --ca-dn CN=${USE_CN} --days ${USE_EXPIRATION_IN_DAYS} --in /app/instances.yaml --out /app/${var_es_keystore_docker_name}.zip --pass ''\""
        echo ""
        cd "${var_cur_dir}" || return
        exit 1
    fi
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        info "copying files from ${var_es_keystore_docker_name}:/app/${var_es_keystore_docker_name}.zip to ${USE_CERT_ZIP_LOC}"
        docker cp "${var_es_keystore_docker_name}:/app/${var_es_keystore_docker_name}.zip" "${USE_CERT_ZIP_LOC}" 2>&1
        var_last_status="$?"
    else
        info " skipped docker cp"
    fi
    if [[ "${var_last_status}" -ne 0 ]]; then
        err "failed to extract /app/${var_es_keystore_docker_name}.zip with command:"
        echo ""
        echo "docker cp \"${var_es_keystore_docker_name}:/app/${var_es_keystore_docker_name}.zip\" \"${USE_CERT_ZIP_LOC}\""
        echo ""
        cd "${var_cur_dir}" || return
        exit 1
    fi
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        cd "${var_use_dirname}" || return
        info "unzipping tls zip: ./${var_es_keystore_docker_name}.zip"
        unzip "./${var_es_keystore_docker_name}.zip" 2>&1
        var_last_status="$?"
        if [[ "${var_last_status}" -ne 0 ]]; then
            err "failed to unzip ./${var_es_keystore_docker_name}.zip with command:"
            echo ""
            echo "unzip ./${var_es_keystore_docker_name}.zip"
            echo ""
            cd "${var_cur_dir}" || return
            exit 1
        fi
    else
        info " skipped unzip: ./${var_es_keystore_docker_name}.zip"
    fi
    if [[ "${var_last_status}" -ne 0 ]]; then
        err "failed to unzip ${var_es_keystore_docker_name}.zip with command:"
        echo ""
        echo "unzip ./${var_es_keystore_docker_name}.zip"
        echo ""
        cd "${var_cur_dir}" || return
        exit 1
    else
        # cleanup
        find . | grep .gitignore | xargs rm -f
        rm -f "./${var_es_keystore_docker_name}.zip" >> /dev/null 2>&1

        extract_tls_assets_from_p12s

        cd "${var_cur_dir}" || return
    fi
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        docker stop "${var_es_keystore_docker_name}" > /dev/null 2>&1 || true
        docker rm -f "${var_es_keystore_docker_name}" > /dev/null 2>&1 || true
    fi
} # generate_new_certs - end

function main() {
    var_last_status="$?"
    if [[ "${var_last_status}" -ne 0 ]]; then
        export GENERATE_NEW_CERTS="1"
    fi

    if [[ "${GENERATE_NEW_CERTS}" -eq 1 ]]; then
        generate_new_certs
        info "generated new certs: ${USE_CERT_ZIP_DIR}"
    else
        info "not generating new certs"
    fi
} # main - end

main

exit 0
