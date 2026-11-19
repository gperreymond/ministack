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
