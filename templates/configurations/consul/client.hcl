{{- if (datasource "config").services.consul.enabled -}}
server = false
{{- end }}
