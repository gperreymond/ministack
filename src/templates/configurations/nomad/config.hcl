{%- if services.nomad.enabled %}
datacenter = "{{ datacenter }}"
data_dir = "/nomad/data"
log_level = "{%- if log_level -%}{{ log_level }}{%- else -%}INFO{%- endif -%}"
log_json = true

leave_on_interrupt = true
leave_on_terminate = true

bind_addr  = "{{ `{{ GetPrivateInterfaces | include \"network\" \"10.1.0.0/24\" | attr \"address\" }}` }}"
{%- endif %}
