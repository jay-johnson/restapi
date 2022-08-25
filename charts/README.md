## Deploying the Rust Rest API helm chart into kubernetes

1.  Customize Credentials Before Deployment

    If you are not using the [external-secrets-operator](https://external-secrets.io/) for secret management, then you can manually set the correct postgres credentials before deploying the chart into the cluster:

    ```bash
    vi ./deploy-tls-and-secrets.sh
    ```

1.  Deploy TLS and Secrets into the Kubernetes Cluster

    ```bash
    ./deploy-tls-and-secrets.sh
    ```

1.  Deploy the Chart

    ```bash
    helm upgrade --install -n default dev-api ./rust-restapi -f ./rust-restapi/values.yaml
    ```

### Cleanup

#### Uninstall the Rest API Chart

```bash
helm delete -n default dev-api
```

#### Delete TLS and Secrets

```bash
kubectl delete secrets -n default dev-api-db-credentials dev-api-jwt-keys dev-api-s3-credentials dev-api-tls dev-pgadmin-tls dev-postgres-db-credentials dev-postgres-tls
```
