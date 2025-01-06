{{- if (datasource "config").services.nomad.enabled -}}
server {
  enabled = true
  bootstrap_expect = {{ (datasource "config").services.nomad.replicas }}
}

client {
  enabled = false
}

autopilot {
  cleanup_dead_servers = true
}
{{- end }}
