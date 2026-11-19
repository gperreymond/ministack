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

ports {
  grpc = 8502
}

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
