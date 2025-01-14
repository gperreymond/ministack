{%- if services.consul.enabled %}
{%- set replicas = 1 -%}
{%- if services.consul.replicas -%}
{%- set replicas = services.consul.replicas -%}
{%- endif -%}
server = true
bootstrap_expect = {{ replicas }}

ui_config {
  enabled = true
}

telemetry {
  disable_hostname = true
  prometheus_retention_time = "30s"
}
{%- endif %}
