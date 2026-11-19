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
{%- if services.consul.enabled -%}
{%- set log_level = "info" -%}
{%- if services.consul.config.log_level -%}{%- set log_level = services.consul.config.log_level -%}{%- endif -%}
datacenter = "{{ datacenter }}"
data_dir   = "/consul/data"
log_level  = "{{ log_level }}"
log_json   = true

# gossip and rpc on the ministack network, api reachable from everywhere (local usage only)
bind_addr   = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
client_addr = "0.0.0.0"

leave_on_terminate = true

retry_join = [
  {%- for item in services.consul.servers %}
  "{{ datacenter }}-{{ item.name }}",
  {%- endfor %}
]

{%- if consul_tls %}
ports {
  http     = -1
  https    = 8501
  grpc     = -1
  grpc_tls = 8503
}
{%- else %}
ports {
  grpc = 8502
}
{%- endif %}

connect {
  enabled = true
}

telemetry {
  disable_hostname          = true
  prometheus_retention_time = "1h"
}
{%- if consul_acl %}

acl {
  enabled                  = true
  default_policy           = "deny"
  enable_token_persistence = true
  tokens {
    # local stack shortcut: every agent uses the management token
    agent = "{{ consul_token }}"
  }
}
{%- endif %}
{%- endif %}
