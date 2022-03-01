#!/bin/bash

var_compose="./compose.yml"
if [[ "${USE_DB_COMPOSE}" != "" ]] && [[ -e "${USE_DB_COMPOSE}" ]]; then
    cp "${USE_DB_COMPOSE}" ./use-compose.yml
    var_compose="./use-compose.yml"
fi

var_test_if_running=$(docker ps -a | grep -c postgres)
if [[ "${var_test_if_running}" -ne 0 ]]; then
    echo "stopping docker stack with:"
    echo "docker-compose -f ${var_compose} down"
    docker-compose -f "${var_compose}" down
fi

echo "starting docker stack with:"
echo "docker-compose -f ${var_compose} up -d"
docker-compose -f "${var_compose}" up -d

exit 0
