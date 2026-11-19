{%- set nomad_acl = false -%}
{%- if services.nomad.config.acl.enabled -%}{%- set nomad_acl = true -%}{%- endif -%}
{%- set nomad_token = "11111111-1111-1111-1111-111111111111" -%}
{%- if services.nomad.config.acl.token -%}{%- set nomad_token = services.nomad.config.acl.token -%}{%- endif -%}
{%- set consul_acl = false -%}
{%- if services.consul.enabled -%}{%- if services.consul.config.acl.enabled -%}{%- set consul_acl = true -%}{%- endif -%}{%- endif -%}
{%- set consul_token = "22222222-2222-2222-2222-222222222222" -%}
{%- if services.consul.config.acl.token -%}{%- set consul_token = services.consul.config.acl.token -%}{%- endif -%}
{%- if services.consul.enabled and services.consul.servers -%}
{%- set bootstrap_expect = services.consul.servers | length -%}
{%- if services.consul.config.server.bootstrap_expect -%}{%- set bootstrap_expect = services.consul.config.server.bootstrap_expect -%}{%- endif -%}
server           = true
bootstrap_expect = {{ bootstrap_expect }}

ui_config {
  enabled = true
}

autopilot {
  cleanup_dead_servers = true
}
{%- if consul_acl %}

acl {
  tokens {
    initial_management = "{{ consul_token }}"
  }
}
{%- endif %}
{%- endif %}
