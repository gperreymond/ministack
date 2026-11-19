{%- if services.consul.enabled -%}
# consul client agent, running next to every nomad agent
server = false
{%- if consul_tls %}

tls {
  defaults {
    ca_file         = "/ministack/tls/ca.pem"
    cert_file       = "/ministack/tls/consul-client.pem"
    key_file        = "/ministack/tls/consul-client-key.pem"
    verify_incoming = false
    verify_outgoing = true
  }
  internal_rpc {
    verify_incoming        = true
    verify_server_hostname = true
  }
}
{%- endif %}
{%- endif %}
