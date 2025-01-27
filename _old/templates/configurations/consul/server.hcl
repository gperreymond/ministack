{%- if services.consul.enabled %}
{%- set replicas = 1 -%}
{%- if services.consul.bootstrap_expect -%}
{%- set replicas = services.consul.bootstrap_expect -%}
{%- endif -%}
server = true
bootstrap_expect = {{ replicas }}

ui_config {
  enabled = true
  {%- if plugins.prometheus.enabled %}
  metrics_provider = "prometheus"
  metrics_proxy {
    base_url = "http://prometheus:9090"
  }
  {%- endif %}
}

telemetry {
  disable_hostname = true
  prometheus_retention_time = "24h"
}
{%- endif %}
