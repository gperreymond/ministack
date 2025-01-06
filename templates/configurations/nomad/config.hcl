{{- if (datasource "config").services.nomad.enabled -}}
datacenter = "{{ (datasource "config").datacenter }}"
data_dir = "/nomad/data"
log_level = "INFO"
log_json = true
leave_on_interrupt = true
leave_on_terminate = true
disable_update_check = true

{{- if (datasource "config").services.consul.enabled }}
consul {
  address = "{{ `{{ GetInterfaceIP \"consul0\" }}` }}:8500"
  server_service_name = "nomad"
  client_service_name = "nomad-client"
  auto_advertise      = true
  server_auto_join    = true
  client_auto_join    = true
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

advertise {
  http = "{{ `{{ GetInterfaceIP \"nomad0\" }}` }}"
  rpc  = "{{ `{{ GetInterfaceIP \"nomad0\" }}` }}"
  serf = "{{ `{{ GetInterfaceIP \"nomad0\" }}` }}"
}

bind_addr = "{{ `{{ GetInterfaceIP \"nomad0\" }}` }}"
{{- end }}
