{{- if (datasource "config").services.nomad.enabled -}}
server {
  enabled = false
}

{{- if (datasource "config").services.consul.enabled }}
consul {
  address = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}:8500"
  grpc_address = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}:8502"
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