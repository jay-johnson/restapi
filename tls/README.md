## Generating TLS Assets with CFSSL

To generate tls assets with these tools, please install:

- [cfssl](https://github.com/cloudflare/cfssl)
- [keytool](https://www.digitalocean.com/community/tutorials/how-to-install-java-with-apt-on-ubuntu-18-04)
- [openssl](https://stackoverflow.com/questions/3016956/how-do-i-install-the-openssl-libraries-on-ubuntu)
- [uuidgen](https://stackoverflow.com/questions/17710958/how-do-i-install-uuidgen)

### Create

To create new tls assets including a new private Certificate Authority (CA) run:

```bash
./create-tls-assets.sh
```

Note: Each time you run ``create-tls-assets.sh`` it will not recreate the CA pem or private key file. Instead it will reuse the existing CA to create new tls assets.

### Deploy

Deployment requires you have a running kubernetes cluster with ``kubectl`` installed locally.

Deploy into kubernetes with:

```bash
./deploy-tls-assets.sh -e dev
```

#### Verify API TLS Assets

##### Cert

```bash
kubectl get secret -n dev -o yaml tls-api-server | grep api-crt.pem | awk '{print $2}' | base64 -d | openssl x509 -text
```

##### CA

```bash
kubectl get secret -n dev -o yaml tls-api-server | grep api-ca.pem | awk '{print $2}' | base64 -d | openssl x509 -text
```

#### Verify Kafka TLS Assets

##### Cert

```bash
kubectl get secret -n dev -o yaml tls-kafka-cluster-0-server | grep kafka-cluster-0-crt.pem | awk '{print $2}' | base64 -d | openssl x509 -text
```

##### CA

```bash
kubectl get secret -n dev -o yaml tls-kafka-cluster-0-server | grep kafka-cluster-0-ca.pem | awk '{print $2}' | base64 -d | openssl x509 -text
```
