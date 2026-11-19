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
{%- endif %}
