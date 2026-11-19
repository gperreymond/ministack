{%- if services.consul.enabled and services.consul.servers -%}
{%- set bootstrap_expect = services.consul.servers | length -%}
{%- if services.consul.config.server.bootstrap_expect -%}{%- set bootstrap_expect = services.consul.config.server.bootstrap_expect -%}{%- endif -%}
server           = true
bootstrap_expect = {{ bootstrap_expect }}

ui_config {
  enabled = true
}

autopilot {
  cleanup_dead_servers = true
}
{%- if consul_acl %}

acl {
  tokens {
    initial_management = "{{ consul_token }}"
  }
}
{%- endif %}
{%- if consul_tls %}

tls {
  defaults {
    ca_file         = "/ministack/tls/ca.pem"
    cert_file       = "/ministack/tls/consul-server.pem"
    key_file        = "/ministack/tls/consul-server-key.pem"
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
