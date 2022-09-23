#!/bin/bash

var_compose="./compose.yml"

# cfssl defaults to 600 permissions which fail pgadmin
# when mounting in podman
if [[ -e ../../tls/pgadmin/server-key.pem ]]; then
    chmod 664 ../../tls/pgadmin/server-key.pem
else
    echo "please create the tls assets - missing pgadmin: 'cd ../../tls && ./create-tls-assets.sh cd ../../docker/db' then retry"
    exit 1
fi

var_test_if_running=$(podman ps -a | grep -c postgres)
if [[ "${var_test_if_running}" -ne 0 ]]; then
    echo "stopping stack with:"
    echo "podman-compose -f ${var_compose} down"
    podman-compose -f "${var_compose}" down
fi

echo "starting stack with:"
echo "podman-compose -f ${var_compose} up -d"
podman-compose -f "${var_compose}" up -d

exit 0
