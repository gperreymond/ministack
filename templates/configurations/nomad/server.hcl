{{- if (datasource "config").services.nomad.enabled -}}
datacenter = "europe-paris"

data_dir = "/nomad/data"
log_level = "INFO"
log_json = true
leave_on_interrupt = true
leave_on_terminate = true
disable_update_check = true

autopilot {
  cleanup_dead_servers = true
}

server {
  enabled = true
  bootstrap_expect = 3
}

client {
  enabled = false
}

consul {
  address = "{{ `{{ GetInterfaceIP \"consul0\" }}` }}:8500"
  server_service_name = "nomad"
  client_service_name = "nomad-client"
  auto_advertise      = true
  server_auto_join    = true
  client_auto_join    = true
}

advertise {
  http = "{{ `{{ GetInterfaceIP \"nomad0\" }}` }}"
  rpc  = "{{ `{{ GetInterfaceIP \"nomad0\" }}` }}"
  serf = "{{ `{{ GetInterfaceIP \"nomad0\" }}` }}"
}

bind_addr = "{{ `{{ GetInterfaceIP \"nomad0\" }}` }}"
{{- end }}
