ALTER SYSTEM SET ssl_cert_file TO '/var/lib/postgresql/certs/postgres.crt';
ALTER SYSTEM SET ssl_key_file TO '/var/lib/postgresql/certs/postgres.key';
ALTER SYSTEM SET ssl_ca_file TO '/var/lib/postgresql/certs/postgres-ca.pem';
ALTER SYSTEM SET ssl TO 'ON';
