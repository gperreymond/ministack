{{- if (datasource "config").services.nomad.enabled -}}
datacenter = "{{ (datasource "config").datacenter }}"
data_dir = "/nomad/data"
log_level = "INFO"
log_json = true
leave_on_interrupt = true
leave_on_terminate = true
disable_update_check = true

advertise {
  http = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}"
  rpc  = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}"
  serf = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}"
}

bind_addr = "{{ `{{ GetInterfaceIP \"ministack0\" }}` }}"
{{- end }}
