{%- set log_level = "info" -%}
{%- if  services.nomad.config.log_level -%}{%- set log_level = services.nomad.config.log_level -%}{%- endif -%}
{%- set bind_addr = "eth0" -%}
{%- if  services.nomad.config.bind_addr -%}{%- set bind_addr = services.nomad.config.bind_addr -%}{%- endif -%}

{%- if 
  services.nomad.servers or
  services.nomad.clients
%}
log_level = "{{ log_level }}"
log_json = true

leave_on_interrupt = true
leave_on_terminate = true

telemetry {
  prometheus_metrics = true
  publish_allocation_metrics = true
  publish_node_metrics = true
}

{%- if bind_addr == "eth0" %}
bind_addr = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
{%- else %}
bind_addr = "{{ bind_addr }}"
{%- endif %}
{%- endif %}
