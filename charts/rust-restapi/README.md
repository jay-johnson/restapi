# Deploy the Rust restapi Chart into Kubernetes using Helm

This guide deploys the chart into the ``dev`` namespace

## Verify Kubernetes Secrets are Ready

```bash
kubectl get secrets -n dev | grep -v NAME | awk '{print $1}'
dev-api-db-credentials
dev-api-jwt-keys
dev-api-s3-credentials
dev-clients-ca
dev-clients-ca-cert
dev-cluster-ca
dev-cluster-ca-cert
tls-api-client
tls-api-peer
tls-api-server
tls-kafka-cluster-0-client
tls-kafka-cluster-0-peer
tls-kafka-cluster-0-server
tls-pgadmin-client
tls-pgadmin-peer
tls-pgadmin-server
tls-postgres-client
tls-postgres-peer
tls-postgres-server
```

If you do not see something similar for your kubernetes namespace, please make sure to run the ``deploy-jwt-and-tls-assets.sh`` from the root of the repo with the ``-e default`` flag:

https://github.com/jay-johnson/restapi/blob/main/deploy-jwt-and-tls-assets.sh

## Deploy the rust-restapi Helm Chart

When the kubernetes assets are ready, you can deploy the restapi chart with:

```bash
helm upgrade --install --create-namespace -n dev dev-api rust-restapi -f ./rust-restapi/values.yaml
```

## Get the Pods

```bash
kubectl get pods -n default
```
