# Sensitive path blocklist (initial)

```
.env
*.pem
*.key
*.pfx
*.p12
id_rsa*
credentials*
secrets*
terraform.tfstate*
.azure/
.aws/
.ssh/
```

Always combine with `.gitignore` and `.openfamiliarignore`.