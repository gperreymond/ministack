{%- set nomad_tls = false -%}
{%- if services.nomad.enabled -%}{%- if services.nomad.config.tls -%}{%- set nomad_tls = true -%}{%- endif -%}{%- endif -%}
{%- set consul_tls = false -%}
{%- if services.consul.enabled -%}{%- if services.consul.config.tls -%}{%- set consul_tls = true -%}{%- endif -%}{%- endif -%}
{%- set vault_tls = false -%}
{%- if services.vault.enabled -%}{%- if services.vault.config.tls -%}{%- set vault_tls = true -%}{%- endif -%}{%- endif -%}
{%- set any_tls = nomad_tls or consul_tls or vault_tls -%}
{%- set nomad_scheme = "http" -%}{%- if nomad_tls -%}{%- set nomad_scheme = "https" -%}{%- endif -%}
{%- set consul_scheme = "http" -%}{%- set consul_port = 8500 -%}
{%- if consul_tls -%}{%- set consul_scheme = "https" -%}{%- set consul_port = 8501 -%}{%- endif -%}
{%- set vault_scheme = "http" -%}{%- if vault_tls -%}{%- set vault_scheme = "https" -%}{%- endif -%}
{%- set nomad_acl = false -%}
{%- if services.nomad.config.acl.enabled -%}{%- set nomad_acl = true -%}{%- endif -%}
{%- set nomad_token = "11111111-1111-1111-1111-111111111111" -%}
{%- if services.nomad.config.acl.token -%}{%- set nomad_token = services.nomad.config.acl.token -%}{%- endif -%}
{%- set consul_acl = false -%}
{%- if services.consul.enabled -%}{%- if services.consul.config.acl.enabled -%}{%- set consul_acl = true -%}{%- endif -%}{%- endif -%}
{%- set consul_token = "22222222-2222-2222-2222-222222222222" -%}
{%- if services.consul.config.acl.token -%}{%- set consul_token = services.consul.config.acl.token -%}{%- endif -%}
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
