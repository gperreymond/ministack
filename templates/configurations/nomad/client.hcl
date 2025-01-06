{{- if (datasource "config").services.nomad.enabled -}}
datacenter = "europe-paris"

data_dir = "/nomad/data"
log_level = "INFO"
log_json = true
leave_on_interrupt = true
leave_on_terminate = true
disable_update_check = true

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