## Generate TLS Assets

This is for quickly building tls assets for this demo, it's not for production. It also uses the Elasticsearch docker image on Elasticsearch's docker registry [docker.elastic.co/elasticsearch/elasticsearch:7.10.2](https://www.docker.elastic.co/r/elasticsearch/elasticsearch:7.10.2). It requires docker, bash and openssl.

1.  Set up tls config

    [./configs/certs.yaml](./configs/certs.yaml)

1.  Generate tls server certificate, server key, certificate authority

    Dry run enabled by default

    ```bash
    ./generate-tls-assets.sh -c ./configs/dev-network.yml
    ```

    Generate assets

    ```bash
    ./generate-tls-assets.sh -f -c ./configs/dev-network.yml
    ```

### Review TLS Assets

1.  API Assets

    Certificate

    ```bash
    openssl x509 -in ./tls/api/api.crt -text
    ```

    Certificate Authority

    ```bash
    openssl x509 -in ./tls/api/api-ca.pem -text
    ```

1.  Postgres Assets

    Certificate

    ```bash
    openssl x509 -in ./tls/postgres/postgres.crt -text
    ```

    Certificate Authority

    ```bash
    openssl x509 -in ./tls/postgres/postgres-ca.pem -text
    ```
