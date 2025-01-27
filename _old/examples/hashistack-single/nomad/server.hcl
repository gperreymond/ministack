server {
  encrypt = "2KNSnCDjE583rXt09x8kZnyBgS7JHPr0FltMXUYvkQA="
}

tls {
  http = true
  rpc  = true

  ca_file   = "/certs/nomad-agent-ca.pem"
  cert_file = "/certs/global-server-nomad.pem"
  key_file  = "/certs/global-server-nomad-key.pem"

  verify_server_hostname = false
  verify_https_client    = false
}
