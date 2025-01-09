{{- if (datasource "config").services.nomad.enabled -}}
{{- if (datasource "config").services.consul.enabled }}
consul {
  address = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}:8500"
}
{{- else }}
server_join {
  retry_max = 3
  retry_interval = "15s"
  retry_join = [
    {{- range seq 1 (datasource "config").services.nomad.replicas }}
    "nomad-server-{{ . }}",
    {{- end }}
  ]
}
{{- end }}

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
