{%- if services.nomad.enabled %}
datacenter = "{{ datacenter }}"
data_dir = "/nomad/data"
log_level = "INFO"
log_json = true

leave_on_interrupt = true
leave_on_terminate = true

advertise {
  http = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
  rpc  = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
  serf = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
}

bind_addr = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
{%- endif %}
