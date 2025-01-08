{{- if (datasource "config").services.nomad.enabled -}}
server {
  enabled = false
}

client {
  enabled = true
  template {
    disable_file_sandbox = true
  }
}

plugin "docker" {
  config {
    allow_privileged = true
    allow_caps = ["all"]
    volumes {
      enabled = true
    }
    extra_labels = ["job_name", "task_group_name", "task_name", "namespace", "node_name"]
  }
}
{{- end }}