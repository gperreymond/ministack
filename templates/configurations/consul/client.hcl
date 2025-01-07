{{- if (datasource "config").services.consul.enabled -}}
server = false

ports {
  grpc = 8502
}

connect {
  enabled = true
}
{{- end }}
