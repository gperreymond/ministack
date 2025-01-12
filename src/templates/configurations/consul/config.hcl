{%- if services.consul.enabled %}
datacenter = "{{ datacenter }}"
data_dir = "/consul/data"
log_level = "{%- if log_level -%}{{ log_level }}{%- else -%}INFO{%- endif -%}"
log_json = true

ports {
  grpc = 8502
  dns = 8600
}

retry_join = [
  {%- for i in range(start=1, end=services.consul.replicas+1) %}
  "consul-server-{{ i }}",
  {%- endfor %}
]

advertise_addr = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
client_addr = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"

bind_addr = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
{%- endif %}