{{- if (datasource "config").services.consul.enabled -}}
server = false

connect {
  enabled = true
}
{{- end }}
