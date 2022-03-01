## Build and Deploy a Secured Postgres Backend

### Build your own tls assets

Please refer to the [Generate TLS Assets - README.md guide](../../certs/README.md)

If you have not generated the certs locally, then you can run this one-line command:

```bash
cd ../../certs && ./generate-tls-assets.sh -c ./configs/dev-network.yml -f && cd ..
```

### Build the Postgres image

Please note: the [Postgres Dockerfile](./postgres/Dockerfile) will copy the tls assets from the ``../../certs/tls`` into a image named: ``localbuild/postgres14``

<a href="https://asciinema.org/a/473134?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473134.png"/></a>

```bash
cd ./docker/db
./build-postgres.sh
```

### Start

Before starting the stack, please note this command will start 2 docker images:

- `localbuild/postgres14` - built from open source [postgres:14.1-alpine](https://hub.docker.com/_/postgres?tab=tags&page=1&name=14.1) image)
- [dpage/pgadmin4](https://hub.docker.com/r/dpage/pgadmin4/) - useful db management web app, but please do not allow access to this docker container access to the public internet - it is not intended to be secure enough for internet traffic

```bash
./start.sh
```

### Verify docker containers are running

```bash
docker ps
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

The default password is ``postgres``, and you can change it in the [compose.yml](./compose.yml) under the ``environment`` variables list:

- ``POSTGRES_USER``
- ``POSTGRES_PASSWORD``
- ``PGADMIN_DEFAULT_PASSWORD``

<a href="https://asciinema.org/a/473135?autoplay=1" width="600" height="400" target="_blank"><img src="https://asciinema.org/a/473135.png"/></a>

```bash
./init-db.sh
```

### Verify db schema

```bash
psql --set=sslmode=require -h 0.0.0.0 -p 5432 -U postgres -d mydb -c "\dt"
```
