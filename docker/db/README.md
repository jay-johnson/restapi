## Start the Postgres Database with TLS for Encryption in Transit and pgAdmin4 with Podman

This approach requires [podman](https://podman.io/getting-started/installation) and [podman-compose](https://github.com/containers/podman-compose):

```bash
./start.sh
```

### Verify Containers are Running

```bash
podman ps -a
```

### Verify connectivity

#### postgres

Postgres is listening to secure connections on tcp port 5432.

##### Verify tls encryption

```bash
openssl s_client -connect 0.0.0.0:5432 -starttls postgres
```

##### Connect to Postgres with psql and require tls encryption

```bash
psql --set=sslmode=require -h 0.0.0.0 -p 5432 -U postgres -d mydb
```

#### pgadmin4

By default, you can login to pgadmin4 at:

Note: unless you add the Certificate Authority to your host, your browser will get an **ssl warning**:

https://0.0.0.0:5433/login?next=%2F

Default login credentials which you can change in the [compose.yml](./compose.yml) under the ``environment`` variables list:

- ``PGADMIN_SETUP_EMAIL``
- ``PGADMIN_SETUP_PASSWORD``

- **Email Address / Username** - ``user@domain.com``
- **Password** - ``123321``

### Initialize the db schema

The default password is ``postgres``, and you can change it in the [env/postgres.env](./env/postgres.env) file.

```bash
./init-db.sh
```

### Verify db schema

```bash
psql --set=sslmode=require -h 0.0.0.0 -p 5432 -U postgres -d mydb -c "\dt"
```
