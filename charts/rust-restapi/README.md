#### Port Forward

```bash
kubectl port-forward -n default svc/rust-restapi 8080:3000 --address 0.0.0.0
```
