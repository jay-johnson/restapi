#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

export VERBOSE="1"
export DATE_FMT="%Y-%m-%dT%H:%M:%SZ"

# debug - only print if VERBOSE != 0
function debug() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} DEBUG ${*}"
    if [[ ${VERBOSE} -ne 0 ]]; then
        echo "${log_str}"
    fi
} # debug - end

function info() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} INFO ${*}"
    if [[ ${VERBOSE} -ne 0 ]]; then
        echo "${log_str}"
    fi
} # info - end

function err() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} ERROR ${*}"
    red "${log_str}"
} # err - end

function trace() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} TRACE ${*}"
    if [[ ${VERBOSE} -ne 0 ]]; then
        echo "${log_str}"
    fi
} # trace - end

function crit() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} CRITICAL ${*}"
    warn "${log_str}"
} # crit - end

function good() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} SUCCESS ${*}"
    if [[ ${SILENT} -eq 0 ]]; then
        green "${log_str}"
    fi
} # good - end

function banner_log() {
    cur_date="$(date -u +"${DATE_FMT}")"
    local log_str="${cur_date} HEADER ${*}"
    yellow "${log_str}"
} # banner_log - end

function init_db() {
    info "init_db - begin"
    var_use_sql_file="${DB_SQL_INIT_FILE}"
    if [[ "${var_use_sql_file}" == "" ]]; then
        var_use_sql_file="./sql/init.sql"
    fi
    if [[ ! -e "${var_use_sql_file}" ]]; then
        err "failed to find the sql file: ${var_use_sql_file}"
        echo ""
        echo "  change this with:"
        echo ""
        echo "  export var_use_sql_file=\"${var_use_sql_file}\""
        echo ""
        exit 1
    fi
    # v2 need to support flexible db name
    var_use_db_name="${POSTGRES_DB_NAME}"
    if [[ "${var_use_db_name}" == "" ]]; then
        var_use_db_name="mydb"
    fi
    yellow "please enter the postgres user password to apply the sql file ${var_use_sql_file} to the db: ${var_use_db_name}" 
    var_command="psql --set=sslmode=require -h localhost -p 5432 -U postgres -d ${var_use_db_name} -f ${var_use_sql_file}"
    info "init_db - running command: ${var_command}"
    eval "${var_command}"
    var_last_command="$?"
    if [[ "${var_last_command}" -ne 0 ]]; then
        err "init_db - failed running command:"
        echo ""
        echo "${var_last_command}"
        echo ""
        exit 1
    fi

    info "init_db - end"
} # init_db - end

init_db

exit 0
