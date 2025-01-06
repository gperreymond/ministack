{{- if (datasource "config").services.nomad.enabled -}}
server {
  enabled = false
}

client {
  enabled = true
}

plugin "docker" {
  config {
    allow_privileged = true
    allow_caps = ["all"]
    volumes {
      enabled = true
    }
  }
}
{{- end }}