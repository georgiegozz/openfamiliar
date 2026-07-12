# Synthetic — not deployable infrastructure for any real account.
terraform {
  required_version = ">= 1.5.0"
}

resource "null_resource" "demo" {
  triggers = {
    purpose = "openfamiliar-context-demo"
  }
}