# database image
FROM postgres:14.1-alpine

# requires building the certs locally with:
# ./docker/db/build-postgres.sh
#
COPY ./tls/postgres/postgres-ca.pem /var/lib/postgresql
COPY ./tls/postgres/postgres.crt /var/lib/postgresql
COPY ./tls/postgres/postgres.key /var/lib/postgresql

RUN chown 70:70 /var/lib/postgresql/postgres.key && chmod 600 /var/lib/postgresql/postgres.key
RUN chown 70:70 /var/lib/postgresql/postgres.crt && chmod 640 /var/lib/postgresql/postgres.crt
RUN chown 70:70 /var/lib/postgresql/postgres-ca.pem && chmod 640 /var/lib/postgresql/postgres-ca.pem

ENTRYPOINT ["docker-entrypoint.sh"] 

# CMD ["-c", "ssl=off"]
CMD ["-l", "-c", "ssl=on", "-c", "ssl_cert_file=/var/lib/postgresql/postgres.crt", "-c", "ssl_key_file=/var/lib/postgresql/postgres.key", "-c", "ssl_ca_file=/var/lib/postgresql/postgres-ca.pem"]
