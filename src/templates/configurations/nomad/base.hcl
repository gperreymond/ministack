{%- set nomad_acl = false -%}
{%- if services.nomad.config.acl.enabled -%}{%- set nomad_acl = true -%}{%- endif -%}
{%- set nomad_token = "11111111-1111-1111-1111-111111111111" -%}
{%- if services.nomad.config.acl.token -%}{%- set nomad_token = services.nomad.config.acl.token -%}{%- endif -%}
{%- set consul_acl = false -%}
{%- if services.consul.enabled -%}{%- if services.consul.config.acl.enabled -%}{%- set consul_acl = true -%}{%- endif -%}{%- endif -%}
{%- set consul_token = "22222222-2222-2222-2222-222222222222" -%}
{%- if services.consul.config.acl.token -%}{%- set consul_token = services.consul.config.acl.token -%}{%- endif -%}
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
{%- if nomad_acl %}

# the management token is set by the bootstrap service (nomad acl bootstrap)
acl {
  enabled = true
}
{%- endif %}

{%- if services.consul.enabled %}

# local consul client agent: service registration, connect, auto join
consul {
  address      = "127.0.0.1:8500"
  grpc_address = "127.0.0.1:8502"
  {%- if consul_acl %}
  # token of the nomad agent itself, local stack shortcut: the management token
  token        = "{{ consul_token }}"
  {%- endif %}
}
{%- endif %}

{%- if bind_addr == "eth0" %}
bind_addr = "{{ `{{ GetInterfaceIP \"eth0\" }}` }}"
{%- else %}
bind_addr = "{{ bind_addr }}"
{%- endif %}
{%- endif %}
