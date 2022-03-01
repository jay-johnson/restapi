## How to build JWT private and public keys for the jsonwebtokens crate

The default algorithm the jsonwebtokens crate is using is ``ECDSA`` with ``SHA-256``.

Here's some more information on why:

[HMAC vs ECDSA for jwt](https://crypto.stackexchange.com/questions/30657/hmac-vs-ecdsa-for-jwt)

### Generate new ECDSA with SHA-256 public and private keys

```bash
openssl ecparam -name prime256v1 -genkey -out private-key.pem
openssl pkcs8 -topk8 -nocrypt -in private-key.pem -out private-key-pkcs8.pem
openssl ec -in private-key.pem -pubout -out public-key.pem
```
