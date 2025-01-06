{{- if (datasource "config").services.consul.enabled -}}
server = true
bootstrap_expect = {{ (datasource "config").services.consul.replicas }}

connect {
  enabled = true
}

ui_config {
  enabled = true
}

telemetry {
  disable_hostname = true
  prometheus_retention_time = "30s"
}
{{- end }}
