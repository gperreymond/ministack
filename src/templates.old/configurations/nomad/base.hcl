{%- if services.nomad.enabled %}
{%- set log_level = "info" -%}
{%- if services.nomad.log_level -%}
{%- set log_level = services.nomad.log_level -%}
{%- endif -%}
datacenter = "{{ datacenter }}"
data_dir = "/nomad/data"
log_level = "{{ log_level }}"
log_json = true

leave_on_interrupt = true
leave_on_terminate = true

telemetry {
  prometheus_metrics = true
  publish_allocation_metrics = true
  publish_node_metrics = true
}

{%- if services.nomad.customized.bind_addr %}
bind_addr = "{{ services.nomad.customized.bind_addr }}"
{%- else %}
bind_addr  = "{{ `{{ GetPrivateInterfaces | include \"network\" \"10.1.0.0/24\" | attr \"address\" }}` }}"
{%- endif %}
{%- endif %}
