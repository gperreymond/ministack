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
{%- endif %}
