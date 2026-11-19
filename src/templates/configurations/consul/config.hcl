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
