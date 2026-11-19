{%- set nomad_tls = false -%}
{%- if services.nomad.enabled -%}{%- if services.nomad.config.tls -%}{%- set nomad_tls = true -%}{%- endif -%}{%- endif -%}
{%- set consul_tls = false -%}
{%- if services.consul.enabled -%}{%- if services.consul.config.tls -%}{%- set consul_tls = true -%}{%- endif -%}{%- endif -%}
{%- set vault_tls = false -%}
{%- if services.vault.enabled -%}{%- if services.vault.config.tls -%}{%- set vault_tls = true -%}{%- endif -%}{%- endif -%}
{%- set any_tls = nomad_tls or consul_tls or vault_tls -%}
{%- set nomad_scheme = "http" -%}{%- if nomad_tls -%}{%- set nomad_scheme = "https" -%}{%- endif -%}
{%- set consul_scheme = "http" -%}{%- set consul_port = 8500 -%}
{%- if consul_tls -%}{%- set consul_scheme = "https" -%}{%- set consul_port = 8501 -%}{%- endif -%}
{%- set vault_scheme = "http" -%}{%- if vault_tls -%}{%- set vault_scheme = "https" -%}{%- endif -%}
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
  {%- if consul_tls %}
  address      = "127.0.0.1:8501"
  ssl          = true
  ca_file      = "/ministack/tls/ca.pem"
  grpc_address = "127.0.0.1:8503"
  grpc_ca_file = "/ministack/tls/ca.pem"
  {%- else %}
  address      = "127.0.0.1:8500"
  grpc_address = "127.0.0.1:8502"
  {%- endif %}
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
