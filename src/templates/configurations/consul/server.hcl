{%- if services.consul.enabled %}
server = true
bootstrap_expect = {{ services.consul.replicas }}

ui_config {
  enabled = true
}

telemetry {
  disable_hostname = true
  prometheus_retention_time = "30s"
}
{%- endif %}
