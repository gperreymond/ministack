{{- if (datasource "config").services.consul.enabled -}}
datacenter = "{{ (datasource "config").datacenter }}"
data_dir = "/consul/data"
log_level = "INFO"
log_json = true

encrypt = "{{ .Env.CONSUL_ENCRYPT_KEY }}"

retry_join = [
  {{- range seq 1 (datasource "config").services.consul.replicas }}
  "consul-server-{{ . }}",
  {{- end }}
]

advertise_addr = "{{ `{{ GetInterfaceIP \"consul0\" }}` }}"
bind_addr = "{{ `{{ GetInterfaceIP \"consul0\" }}` }}"
client_addr = "{{ `{{ GetInterfaceIP \"consul0\" }}` }}"
{{- end }}