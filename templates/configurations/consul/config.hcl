{{- if (datasource "config").services.consul.enabled -}}
datacenter = "{{ (datasource "config").datacenter }}"
data_dir = "/consul/data"
log_level = "INFO"
log_json = true

ports {
  grpc = 8502
  dns = 8600
}

tls {
  defaults {
    verify_incoming = false
    verify_outgoing = false
    verify_server_hostname = false
  }
}

retry_join = [
  {{- range seq 1 (datasource "config").services.consul.replicas }}
  "consul-server-{{ . }}",
  {{- end }}
]

advertise_addr = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}"
bind_addr = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}"
client_addr = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}"
{{- end }}