tls {
  http = true
  rpc  = true

  ca_file   = "/certs/nomad-agent-ca.pem"
  cert_file = "/certs/global-client-nomad.pem"
  key_file  = "/certs/global-client-nomad-key.pem"

  verify_server_hostname = false
  verify_https_client    = false
}
