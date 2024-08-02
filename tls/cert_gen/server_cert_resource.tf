resource "tls_private_key" "server_key" {
  algorithm = "RSA"
  rsa_bits  = 4096
}

resource "tls_cert_request" "server_cert_req" {
  key_algorithm   = tls_private_key.server_key.algorithm
  private_key_pem = tls_private_key.server_key.private_key_pem

  subject {
    common_name = "TidyBee gRPC Test Server Cert"
  }

  dns_names = ["localhost", "*.test.google.fr", "dev.tidybee.com"]
}

resource "tls_locally_signed_cert" "server_cert" {
  cert_request_pem = tls_cert_request.server_cert_req
  ca_key_algorithm = tls_private_key.root.algorithm
  ca_private_key_pem = tls_private_key.root.private_key_pem
  ca_cert_pem = tls_self_signed_cert.root.cert_pem

  validity_period_hours = 87600
  early_renewal_hours = 8760

  allowed_uses = ["server_auth"]
}

resource "local_file" "cert_file" {
  filename = "../${path.module}/server_cert.pem"
  content  = tls_locally_signed_cert.server_cert.cert_pem
}

resource "local_file" "key_file" {
  filename = "../${path.module}/server_key.pem"
  content  = tls_private_key.server_key.private_key_pem
}