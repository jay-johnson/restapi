## How to build JWT private and public keys for the jsonwebtokens crate

The default algorithm the [jsonwebtoken crate](https://crates.io/crates/jsonwebtoken) is using is ``ECDSA`` with ``SHA-256``.

### Generate new ECDSA with SHA-256 public and private signing keys

```bash
openssl ecparam -name prime256v1 -genkey -out private-key.pem
openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out private-key-pkcs8.pem
openssl ec -in private-key.pem -pubout -out public-key.pem
```
