#!/bin/bash
set -e

cat >> "$PGDATA/postgresql.conf" <<-EOCONF
port = 5433
ssl = on
ssl_cert_file = '/var/lib/postgresql/postgres.key'
ssl_key_file = '/var/lib/postgresql/postgres.crt'
ssl_ca_file = '/var/lib/postgresql/postgres-ca.pem'
EOCONF

cat > "$PGDATA/pg_hba.conf" <<-EOCONF
# TYPE  DATABASE        USER            ADDRESS              METHOD
hostssl all             datawriter      0.0.0.0/0            trust
hostssl all             datawriter      ::0/0                trust
host    all             datawriter      0.0.0.0/0            reject
host    all             datawriter      ::0/0                reject

# IPv4 local connections:
host    all             postgres        0.0.0.0/0            trust
# IPv6 local connections:
host    all             postgres        ::0/0                trust
# Unix socket connections:
local   all             postgres                             trust
EOCONF

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
    SET password_encryption TO 'scram-sha-256';
    CREATE ROLE datawriter PASSWORD 'postgres' LOGIN;
    CREATE EXTENSION hstore;
    CREATE EXTENSION citext;
EOSQL
